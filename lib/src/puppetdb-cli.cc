#include <stdio.h>
#include <curl/curl.h>
#include <string>
#include <tuple>
#include <queue>
#include <mutex>
#include <thread>
#include <condition_variable>

#include <rapidjson/reader.h>
#include <rapidjson/prettywriter.h>

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
    LOG_DEBUG("puppetdb-cli version is %1%", PUPPETDB_CLI_VERSION_WITH_COMMIT);
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
    return parse_config(read_config(config_path));
}

PuppetDBConn::PuppetDBConn() :
        server_urls_ { { "http://127.0.0.1:8080" } },
        cacert_ { "" },
        cert_ { "" },
        key_ { "" } {};


PuppetDBConn::PuppetDBConn(const json::JsonContainer& config) :
        server_urls_ { parseServerUrls(config) },
        cacert_ { "" },
        cert_ { "" },
        key_ { "" } {
            if (config.includes("cacert")) {
                cacert_ = config.get<std::string>("cacert");
            }
            if (config.includes("cert")) {
                cert_ = config.get<std::string>("cert");
            }
            if (config.includes("key")) {
                key_ = config.get<std::string>("key");
            }};

string PuppetDBConn::getServerUrl() const {
    return server_urls_.size() ? server_urls_[0] : "";
}

unique_ptr<CURL, function<void(CURL*)> >
PuppetDBConn::getCurlHandle() const {
    auto curl = unique_ptr< CURL, function<void(CURL*)> >(curl_easy_init(),
                                                          curl_easy_cleanup);
    if (cacert_ != "") curl_easy_setopt(curl.get(), CURLOPT_CAINFO, cacert_.c_str());
    if (cert_ != "") curl_easy_setopt(curl.get(), CURLOPT_SSLCERT, cert_.c_str());
    if (key_ != "") curl_easy_setopt(curl.get(), CURLOPT_SSLKEY, key_.c_str());
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

typedef tuple< mutex, condition_variable, queue<int> > JsonQueue;
size_t write_queue(char *ptr, size_t size, size_t nmemb, JsonQueue& stream) {
    const size_t written = size * nmemb;
    unique_lock<mutex> lk(get<0>(stream));
    for (size_t i = 0; i < written; i++) {
        get<2>(stream).push(ptr[i]);
    }
    lk.unlock();
    get<1>(stream).notify_one();
    return written;
}

size_t write_data(char *ptr, size_t size, size_t nmemb, FILE *stream) {
    const size_t written = fwrite(ptr, size, nmemb, stream);
    return written;
}

size_t write_body(char *ptr, size_t size, size_t nmemb, void *userdata){
    return size * nmemb;
}


class JsonQueueWrapper {
  public:
    typedef char Ch;
    JsonQueueWrapper(JsonQueue& stream) : stream_(stream) {}
    Ch Peek() const { // 1
        int c = Front();
        return c == char_traits<char>::eof() ? '\0' : (Ch)c;
    }
    Ch Take() { // 2
        int c = Get();
        return c == char_traits<char>::eof() ? '\0' : (Ch)c;
    }
    size_t Tell() const { return (size_t)get<2>(stream_).size(); } // 3
    Ch* PutBegin() { assert(false); return 0; }
    void Put(Ch) { assert(false); }
    void Flush() { assert(false); }
    size_t PutEnd(Ch*) { assert(false); return 0; }
  private:
    int Front() const {
        unique_lock<mutex> lk(get<0>(stream_));
        get<1>(stream_).wait(lk, [&]{ return !get<2>(stream_).empty(); });
        int c = get<2>(stream_).front();
        lk.unlock();
        return c;
    };
    int Get() {
        unique_lock<mutex> lk(get<0>(stream_));
        get<1>(stream_).wait(lk, [&]{ return !get<2>(stream_).empty(); });
        int c = get<2>(stream_).front();
        get<2>(stream_).pop();
        lk.unlock();
        return c;
    };
    JsonQueueWrapper(const JsonQueueWrapper&);
    JsonQueueWrapper& operator=(const JsonQueueWrapper&);
    JsonQueue& stream_;
};

class OStreamWrapper {
  public:
    typedef char Ch;
    OStreamWrapper(std::ostream& os) : os_(os) {
    }
    Ch Peek() const { assert(false); return '\0'; }
    Ch Take() { assert(false); return '\0'; }
    size_t Tell() const { return 0; }
    Ch* PutBegin() { assert(false); return 0; }
    void Put(Ch c) { os_.put(c); }                  // 1
    void Flush() { os_.flush(); }                   // 2
    size_t PutEnd(Ch*) { assert(false); return 0; }
  private:
    OStreamWrapper(const OStreamWrapper&);
    OStreamWrapper& operator=(const OStreamWrapper&);
    std::ostream& os_;
};

void write_pretty(JsonQueue& stream) {
    rapidjson::Reader reader;
    JsonQueueWrapper is(stream);
    OStreamWrapper os(nowide::cout);
    rapidjson::PrettyWriter<OStreamWrapper> writer(os);
    if (!reader.Parse<rapidjson::kParseValidateEncodingFlag>(is, writer)) {
        nowide::cerr << "error parsing response" << endl;
    }
};


void
pdb_query(const PuppetDBConn& conn,
          const string& query_str) {
    auto curl = conn.getCurlHandle();

    auto url_encoded_query = unique_ptr< char, function<void(char*)> >(
        curl_easy_escape(curl.get(), query_str.c_str(), query_str.length()),
        curl_free);

    const auto server_url = conn.getServerUrl()
            + "/pdb/query/v4?query="
            + url_encoded_query.get();

    JsonQueue stream;

    curl_easy_setopt(curl.get(), CURLOPT_URL, server_url.c_str());
    curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, ref(stream));
    curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_queue);

    thread t1(write_pretty, ref(stream));

    const CURLcode curl_code = curl_easy_perform(curl.get());

    get<2>(stream).push(char_traits<char>::eof());
    get<1>(stream).notify_one();
    t1.join();

    long http_code = 0;
    curl_easy_getinfo(curl.get(), CURLINFO_RESPONSE_CODE, &http_code);
    if (!(http_code == 200 && curl_code == CURLE_OK)) {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "error: " << curl_easy_strerror(curl_code) << endl;
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
           const string& infile,
           const string& command_versions) {
    auto curl = conn.getCurlHandle();
    const string server_url = conn.getServerUrl() + "/pdb/admin/v1/archive";

    curl_httppost* formpost = NULL;
    curl_httppost* lastptr = NULL;
    curl_formadd(&formpost, &lastptr, CURLFORM_COPYNAME, "archive",
                 CURLFORM_FILE, infile.c_str(), CURLFORM_END);
    curl_formadd(&formpost, &lastptr, CURLFORM_COPYNAME, "command_versions",
                 CURLFORM_COPYCONTENTS, command_versions.c_str(), CURLFORM_END);

    curl_easy_setopt(curl.get(), CURLOPT_URL, server_url.c_str());
    curl_easy_setopt(curl.get(), CURLOPT_HTTPPOST, formpost);
    curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_body);

    boost::nowide::cout << "Importing " << infile << " to PuppetDB..." << endl;

    const CURLcode curl_code = curl_easy_perform(curl.get());
    long http_code = 0;
    curl_easy_getinfo(curl.get(), CURLINFO_RESPONSE_CODE, &http_code);

    if (http_code == 200 && curl_code == CURLE_OK) {
      nowide::cout << "Finished importing " << infile << " to PuppetDB." << endl;
    } else {
      logging::colorize(nowide::cerr, logging::log_level::fatal);
      nowide::cerr << "error: " << curl_easy_strerror(curl_code) << endl;
      logging::colorize(nowide::cerr);
    }

    curl_formfree(formpost);
}

}  // puppetdb
