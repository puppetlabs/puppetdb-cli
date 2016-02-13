#include <stdio.h>
#include <curl/curl.h>
#include <string>
#include <queue>

#include <rapidjson/reader.h>
#include <rapidjson/prettywriter.h>
#include <rapidjson/filewritestream.h>


#define BOOST_THREAD_PROVIDES_GENERIC_SHARED_MUTEX_ON_WIN
#include <boost/thread.hpp>
#include <boost/thread/mutex.hpp>
#include <boost/thread/locks.hpp>
#include <boost/thread/lock_types.hpp>
#include <boost/thread/condition_variable.hpp>
#include <boost/utility/string_ref.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/nowide/iostream.hpp>
#include <boost/nowide/args.hpp>
#include <boost/filesystem.hpp>

#include <leatherman/logging/logging.hpp>
#include <leatherman/json_container/json_container.hpp>
#include <leatherman/file_util/file.hpp>

#include <puppetdb-cli/version.h>
#include <puppetdb-cli/puppetdb-cli.hpp>

namespace puppetdb {

using namespace std;
namespace fs = boost::filesystem;
namespace nowide = boost::nowide;
namespace json = leatherman::json_container;
namespace futil = leatherman::file_util;
namespace logging = leatherman::logging;

string
version()
{
    LOG_DEBUG("puppetdb-cli version is {1}", PUPPETDB_CLI_VERSION_WITH_COMMIT);
    return PUPPETDB_CLI_VERSION_WITH_COMMIT;
}

string
read_config(const string& config_path) {
    const string expanded_config_path
    { futil::tilde_expand("~/.puppetlabs/client-tools/puppetdb.conf") };
    return fs::exists(expanded_config_path) ?
            futil::read(expanded_config_path) : "";
}

PuppetDBConn
parse_config(const string& config_content) {
    const json::JsonContainer raw_config(config_content);
    const auto puppetdb_conn = raw_config.includes("puppetdb") ?
            PuppetDBConn(raw_config.get<json::JsonContainer>("puppetdb")):
            PuppetDBConn();
    if (puppetdb_conn.getServerUrl().empty()) {
        throw std::runtime_error { "invalid `server_urls` in configuration" };
    }
    return puppetdb_conn;
}

PuppetDBConn
get_puppetdb(const string& config_path) {
    const auto config_content = read_config(config_path);
    return config_content.empty() ? PuppetDBConn() : parse_config(config_content);
}

PuppetDBConn::PuppetDBConn() :
        server_urls_ { { "http://127.0.0.1:8080" } },
        cacert_ { "" },
        cert_ { "" },
        key_ { "" } {};


PuppetDBConn::PuppetDBConn(const json::JsonContainer& config) :
        server_urls_ { parseServerUrls(config) },
        cacert_ { config.getWithDefault<std::string>("cacert", "") },
        cert_ { config.getWithDefault<std::string>("cert", "") },
        key_ { config.getWithDefault<std::string>("key", "") } {};

string PuppetDBConn::getServerUrl() const {
    return server_urls_.size() ? server_urls_[0] : "";
}

unique_ptr<CURL, function<void(CURL*)> >
PuppetDBConn::getCurlHandle() const {
    auto curl = unique_ptr< CURL, function<void(CURL*)> >(curl_easy_init(),
                                                          curl_easy_cleanup);
    if (!cacert_.empty()) curl_easy_setopt(curl.get(), CURLOPT_CAINFO, cacert_.c_str());
    if (!cert_.empty()) curl_easy_setopt(curl.get(), CURLOPT_SSLCERT, cert_.c_str());
    if (!key_.empty()) curl_easy_setopt(curl.get(), CURLOPT_SSLKEY, key_.c_str());
    return curl;
}

server_urls_t
PuppetDBConn::parseServerUrls(const json::JsonContainer& config) {
    if (config.includes("server_urls")) {
        const auto urls_type = config.type("server_urls");
        if (urls_type == json::DataType::Array) {
            return config.get<server_urls_t>("server_urls");
        } else if (urls_type == json::DataType::String) {
            return { config.get<string>("server_urls") };
        } else {
            return {};
        }
    } else {
        return {"http://127.0.0.1:8080"};
    }
}

class SynchronizedCharQueue {
  public:
    vector<char> front() {
        boost::unique_lock<boost::mutex> mlock(mutex_);
        cond_.wait(mlock, [&]{ return !queue_.empty(); });
        return queue_.front();
    }

    vector<char> pop() {
        boost::unique_lock<boost::mutex> mlock(mutex_);
        cond_.wait(mlock, [&]{ return !queue_.empty(); });
        auto item = queue_.front();
        queue_.pop();
        return item;
    }

    void push(vector<char>&& items) {
        boost::unique_lock<boost::mutex> mlock(mutex_);
        queue_.push(items);
        mlock.unlock();
        cond_.notify_one();
    }

    void push(const char& item) {
        boost::unique_lock<boost::mutex> mlock(mutex_);
        queue_.push({item});
        mlock.unlock();
        cond_.notify_one();
    }

  private:
    queue< vector<char> > queue_;
    boost::mutex mutex_;
    boost::condition_variable cond_;
};

struct UserData {
    bool collecting_header;
    vector<string> headers;
    string content_type;
    string error_response;
    SynchronizedCharQueue stream;
};

size_t write_queue(char *ptr, size_t size, size_t nmemb, UserData& userdata) {
    const size_t written = size * nmemb;
    if (userdata.collecting_header) {
        if (written == 2 && ptr[0] == '\r' && ptr[1] == '\n') {
            // We've reached the end of the header
            userdata.collecting_header = false;
            return written;
        } else {
            boost::string_ref header(ptr, written);
            userdata.headers.push_back(header.to_string());

            if (!header.starts_with("HTTP/")) {
                auto pos = header.find_first_of(':');
                if (pos == boost::string_ref::npos) {
                    LOG_WARNING("unexpected HTTP response header: {1}.", header);
                } else {
                    auto name = header.substr(0, pos).to_string();
                    boost::trim(name);
                    if (name == "Content-Type") {
                        auto value = header.substr(pos + 1).to_string();
                        boost::trim(value);
                        userdata.content_type = value;
                    }
                }
            }
        }
    } else {
        userdata.stream.push(vector<char>(ptr, ptr + written));
    }
    return written;
}

size_t write_data(char *ptr, size_t size, size_t nmemb, FILE *stream) {
    const size_t written = fwrite(ptr, size, nmemb, stream);
    return written;
}

size_t write_body(char *ptr, size_t size, size_t nmemb, void *userdata){
    return size * nmemb;
}

// SynchronizedCharQueueWrapper is a rapidjson adapter which satisfies the
// `ReadStream` interface for a rapidjson::Reader. This allows us to parse data
// from curl as we recieve it.
class SynchronizedCharQueueWrapper {
  public:
    typedef char Ch;
    SynchronizedCharQueueWrapper(SynchronizedCharQueue& stream) : stream_(stream),
                                                                  count_(0),
                                                                  buffer_(stream_.pop()),
                                                                  buffer_iterator_(buffer_.begin()) {}
    char Peek() const {
        int c = Front();
        return c == char_traits<char>::eof() ? '\0' : static_cast<char>(c);
    }
    char Take() {
        int c = Get();
        return c == char_traits<char>::eof() ? '\0' : static_cast<char>(c);
    }
    size_t Tell() const { return count_; }
    char* PutBegin() { assert(false); return 0; }
    void Put(char c) { assert(false); }
    void Flush() { assert(false); }
    size_t PutEnd(char* c) { assert(false); return 0; }

  private:
    int Front() const { return *buffer_iterator_; }
    int Get() {
        int c = *buffer_iterator_;
        count_++;
        ++buffer_iterator_;
        if (buffer_iterator_ == buffer_.end()) {
            buffer_ = stream_.pop();
            buffer_iterator_ = buffer_.begin();
        }
        return c;
    }
    SynchronizedCharQueueWrapper(const SynchronizedCharQueueWrapper&);
    SynchronizedCharQueueWrapper& operator=(const SynchronizedCharQueueWrapper&);
    SynchronizedCharQueue& stream_;
    size_t count_;
    vector<char> buffer_;
    vector<char>::iterator buffer_iterator_;
};

void write_pretty(UserData& userdata) {
    // This will make sure to block until we have parsed the header
    userdata.stream.front();

    boost::string_ref content_type{ userdata.content_type };
    if (content_type.starts_with("application/json")) {
        rapidjson::Reader reader;
        SynchronizedCharQueueWrapper is(userdata.stream);
        char writeBuffer[65536];
        rapidjson::FileWriteStream os(stdout, writeBuffer, sizeof(writeBuffer));
        rapidjson::PrettyWriter<rapidjson::FileWriteStream> writer(os);
        if (!reader.Parse<rapidjson::kParseValidateEncodingFlag>(is, writer)) {
            nowide::cerr << "error parsing response" << endl;
        }
    } else {
        vector<char> buffer;
        do {
            buffer = userdata.stream.pop();
        } while (buffer.empty());

        // We always signal the end with a vector of `{ '\0' }`
        while ( static_cast<int>(buffer[0]) != char_traits<char>::eof() ) {
            userdata.error_response.append(buffer.begin(), buffer.end());
            do {
                buffer = userdata.stream.pop();
            } while (buffer.empty());
        }
    }
}

string
convert_query_to_post_data(const string& query_str) {
    // If this is PQL then we need to wrap the query in double-quotes, otherwise
    // the query is AST and we leave it alone
    string query_str_copy { query_str };
    boost::trim(query_str_copy);
    const string query = (query_str_copy[0] == '[') ? query_str:"\""+ query_str +"\"";
    return "{\"query\":" + query + "}";
}

void
pdb_query(const PuppetDBConn& conn,
          const string& endpoint,
          const string& query_str) {
    auto curl = conn.getCurlHandle();
    const string server_url = conn.getServerUrl() + endpoint;
    curl_easy_setopt(curl.get(), CURLOPT_URL, server_url.c_str());

    const string post_data = convert_query_to_post_data(query_str);
    curl_easy_setopt(curl.get(), CURLOPT_POSTFIELDS, post_data.c_str());

    auto headers = unique_ptr<curl_slist, function<void(curl_slist*)> >(NULL,
                                                                        curl_slist_free_all);
    curl_easy_setopt(curl.get(),
                     CURLOPT_HTTPHEADER,
                     curl_slist_append(headers.get(), "Content-Type: application/json"));

    UserData userdata;
    userdata.collecting_header = true;
    curl_easy_setopt(curl.get(), CURLOPT_URL, server_url.c_str());
    curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, &userdata);
    curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_queue);
    curl_easy_setopt(curl.get(), CURLOPT_HEADER, 1L);

    boost::thread t1(write_pretty, boost::ref(userdata));
    const CURLcode curl_code = curl_easy_perform(curl.get());
    if (curl_code != CURLE_OK) {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "error connecting to PuppetDB: "
                     << curl_easy_strerror(curl_code) << endl;
        logging::colorize(nowide::cerr);
        return;
    }

    userdata.stream.push(char_traits<char>::eof());
    t1.join();
    long http_code = 0;
    curl_easy_getinfo(curl.get(), CURLINFO_RESPONSE_CODE, &http_code);
    if (http_code != 200) {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "error status " << http_code << " querying PuppetDB:" << endl;
        nowide::cerr << userdata.error_response << endl;
        logging::colorize(nowide::cerr);
    }
}

void
pdb_export(const PuppetDBConn& conn,
           const string& path,
           const string& anonymization) {
    auto curl = conn.getCurlHandle();
    const string server_url = conn.getServerUrl()
            + "/pdb/admin/v1/archive?anonymization="
            + anonymization;
    curl_easy_setopt(curl.get(), CURLOPT_URL, server_url.c_str());

    auto fp = unique_ptr< FILE, function<void(FILE*)> >(fopen(path.c_str(), "wb"),
                                                        fclose);
    curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, fp.get());
    curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_data);
    nowide::cout << "Exporting PuppetDB..." << endl;
    const CURLcode curl_code = curl_easy_perform(curl.get());
    long http_code = 0;
    curl_easy_getinfo(curl.get(), CURLINFO_RESPONSE_CODE, &http_code);
    if (http_code == 200 && curl_code != CURLE_ABORTED_BY_CALLBACK) {
        nowide::cout << "Finished exporting PuppetDB archive to " << path << "." << endl;
    } else {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "error: failed to download PuppetDB archive" << endl;
        logging::colorize(nowide::cerr);
    }
}

void
pdb_import(const PuppetDBConn& conn,
           const string& infile) {
    auto curl = conn.getCurlHandle();
    const string server_url = conn.getServerUrl() + "/pdb/admin/v1/archive";

    curl_httppost* formpost = NULL;
    curl_httppost* lastptr = NULL;
    curl_formadd(&formpost, &lastptr, CURLFORM_COPYNAME, "archive",
                 CURLFORM_FILE, infile.c_str(), CURLFORM_END);

    curl_easy_setopt(curl.get(), CURLOPT_URL, server_url.c_str());
    curl_easy_setopt(curl.get(), CURLOPT_HTTPPOST, formpost);
    curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_body);

    nowide::cout << "Importing " << infile << " to PuppetDB..." << endl;

    const CURLcode curl_code = curl_easy_perform(curl.get());
    if (curl_code != CURLE_OK) {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "error connecting to PuppetDB: "
                     << curl_easy_strerror(curl_code) << endl;
        logging::colorize(nowide::cerr);

    } else {
        long http_code = 0;
        curl_easy_getinfo(curl.get(), CURLINFO_RESPONSE_CODE, &http_code);
        if (http_code == 200) {
            nowide::cout << "Finished importing " << infile << " to PuppetDB." << endl;
        } else {
            logging::colorize(nowide::cerr, logging::log_level::fatal);
            nowide::cerr << "error status " << http_code
                         << " importing archive to PuppetDB" << endl;
            logging::colorize(nowide::cerr);
        }
    }

    curl_formfree(formpost);
}
}  // puppetdb
