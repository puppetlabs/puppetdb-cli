#include <stdio.h>
#include <curl/curl.h>
#include <string>
#include <sstream>
#include <vector>
#include <queue>

#include <rapidjson/reader.h>
#include <rapidjson/prettywriter.h>
#include <rapidjson/filewritestream.h>

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

const server_urls_t default_server_urls = { "http://127.0.0.1:8080" };

string
version() {
    LOG_DEBUG("puppetdb-cli version is %1%", PUPPETDB_CLI_VERSION_WITH_COMMIT);
    return PUPPETDB_CLI_VERSION_WITH_COMMIT;
}

string
read_config(const string& config_path) {
    const string expanded_config_path
    { futil::tilde_expand(config_path) };
    if (fs::exists(expanded_config_path)) {
        return futil::read(expanded_config_path);
    } else {
        LOG_WARNING("config file %1% does not exist. Using default configuration.", config_path);
        return "";
    }
}

PuppetDBConn
parse_config(const string& config_content,
             const string& urls,
             const SSLCredentials& ssl_creds) {
    const json::JsonContainer raw_config(config_content);
    const auto puppetdb_conn = raw_config.includes("puppetdb") ?
            PuppetDBConn(raw_config.get<json::JsonContainer>("puppetdb"),
                         urls,
                         ssl_creds):
            PuppetDBConn(urls, ssl_creds);
    if (puppetdb_conn.getServerUrl().empty()) {
        throw std::runtime_error { "invalid `server_urls` in configuration" };
    }
    return puppetdb_conn;
}

server_urls_t
parse_server_urls_str(const string& urls) {
    server_urls_t server_urls;
    boost::split(server_urls,
                 urls,
                 boost::is_any_of(","),
                 boost::token_compress_on);
    return server_urls;
}


server_urls_t
parse_server_urls(const json::JsonContainer& config) {
    if (config.includes("server_urls")) {
        const auto urls_type = config.type("server_urls");
        if (urls_type == json::DataType::Array) {
            return config.get<server_urls_t>("server_urls");
        } else if (urls_type == json::DataType::String) {
            return parse_server_urls_str(config.get<string>("server_urls"));
        } else {
            return {};
        }
    } else {
        return default_server_urls;
    }
}

PuppetDBConn
get_puppetdb(const string& config_path,
             const string& urls,
             const SSLCredentials& ssl_creds) {
    const auto config_content = read_config(config_path);
    return config_content.empty() ? PuppetDBConn(urls, ssl_creds)
            : parse_config(config_content, urls, ssl_creds);
}

PuppetDBConn::PuppetDBConn() :
        server_urls_ { default_server_urls } ,
        cacert_ {},
        cert_ {},
        key_ {} {};

PuppetDBConn::PuppetDBConn(const string& urls,
                           const SSLCredentials& ssl_creds) :
        server_urls_ { urls.empty() ?
            default_server_urls :
            parse_server_urls_str(urls)},
        cacert_ { ssl_creds.cacert },
        cert_ { ssl_creds.cert },
        key_ { ssl_creds.key } {};

PuppetDBConn::PuppetDBConn(const json::JsonContainer& config,
                           const string& urls,
                           const SSLCredentials& ssl_creds) :
        server_urls_ { urls.empty() ?
            parse_server_urls(config) :
            parse_server_urls_str(urls)},
        cacert_ { ssl_creds.cacert },
        cert_ { ssl_creds.cert },
        key_ { ssl_creds.key } {
            if (cacert_.empty() && config.includes("cacert")) {
                cacert_ = config.get<std::string>("cacert");
            }
            if (cert_.empty() && config.includes("cert")) {
                cert_ = config.get<std::string>("cert");
            }
            if (key_.empty() && config.includes("key")) {
                key_ = config.get<std::string>("key");
            }};

string PuppetDBConn::getServerUrl() const {
    return server_urls_.size() ? server_urls_[0] : "";
}

SSLCredentials
PuppetDBConn::getSSLCredentials() const {
    return SSLCredentials{ cacert_, cert_, key_ };
}

string
convert_query_to_post_data(const string& query_str) {
    // If this is PQL then we need to wrap the query in double-quotes, otherwise
    // the query is AST and we leave it alone
    string query_str_copy { query_str };
    boost::trim(query_str_copy);
    const string query = (boost::starts_with(query_str_copy, "[")) ?
            query_str : "\""+ query_str +"\"";
    return "{\"query\":" + query + "}";
}


struct error_response_exception : runtime_error {
  public:
    error_response_exception(long status, string response) :
        runtime_error("error status from server."),
        _status(status),
        _response(move(response)) {}

    long status() const { return _status; }
    string const& response() const { return _response; }

 private:
    long _status;
    string _response;
};

struct curl_input_stream {
  public:
    using Ch = char;

    explicit curl_input_stream(const string& url,
                               const string& post_fields,
                               const SSLCredentials& ssl_creds) {
        try {
            _multi_handle = curl_multi_init();
            if (!_multi_handle)
                throw runtime_error("failed to initialize multi handle.");

            _easy_handle = curl_easy_init();
            if (!_easy_handle)
                throw runtime_error("failed to initialize easy handle.");

            if (curl_multi_add_handle(_multi_handle, _easy_handle) != CURLM_OK)
                throw runtime_error("failed to add easy handle to multi handle.");

            if (!ssl_creds.cacert.empty() &&
                curl_easy_setopt(_easy_handle, CURLOPT_CAINFO, ssl_creds.cacert.c_str()) != CURLE_OK)
                throw runtime_error("failed to set ca info option.");
            if (!ssl_creds.cert.empty() &&
                curl_easy_setopt(_easy_handle, CURLOPT_SSLCERT, ssl_creds.cert.c_str()) != CURLE_OK)
                throw runtime_error("failed to set ssl cert option.");
            if (!ssl_creds.key.empty() &&
                curl_easy_setopt(_easy_handle, CURLOPT_SSLKEY, ssl_creds.key.c_str()) != CURLE_OK)
                throw runtime_error("failed to set ssl key option.");

            if (curl_easy_setopt(_easy_handle, CURLOPT_URL, url.c_str()) != CURLE_OK)
                throw runtime_error("failed to set url option.");

            if (curl_easy_setopt(_easy_handle, CURLOPT_POSTFIELDS, post_fields.c_str()) != CURLE_OK)
                throw runtime_error("failed to set post fields option.");

            if (curl_easy_setopt(_easy_handle, CURLOPT_HTTPHEADER,
                                 curl_slist_append(_headers, "Content-Type: application/json")) != CURLE_OK)
                throw runtime_error("failed to set http header option.");

            if (curl_easy_setopt(_easy_handle, CURLOPT_WRITEDATA, this) != CURLE_OK)
                throw runtime_error("failed to set write data option.");

            if (curl_easy_setopt(_easy_handle, CURLOPT_WRITEFUNCTION, process_response) != CURLE_OK)
                throw runtime_error("failed to set write function option.");
        } catch (exception const&) {
            // Clean up anything that was allocated inside this constructor
            this->~curl_input_stream();
            throw;
        }
    }

    ~curl_input_stream() {
        if (_multi_handle) {
            if (_easy_handle) {
                curl_multi_remove_handle(_multi_handle, _easy_handle);
                curl_easy_cleanup(_easy_handle);
                _easy_handle = nullptr;
            }
            curl_multi_cleanup(_multi_handle);
            _multi_handle = nullptr;
        }

        if (_headers)
            curl_slist_free_all(_headers);
    }

    Ch Peek() const {
        const_cast<curl_input_stream&>(*this).read();
        return _offset >= _buffer.size() ? 0 : _buffer[_offset];
    }

    Ch Take() {
        auto current = Peek();
        ++_offset;
        ++_read_count;
        return current;
    }

    size_t Tell() const { return _read_count; }

    Ch* PutBegin() { throw runtime_error("unexpected write to input stream."); }
    void Put(Ch c) { throw runtime_error("unexpected write to input stream."); }
    void Flush() { throw runtime_error("unexpected write to input stream."); }
    size_t PutEnd(Ch* c) { throw runtime_error("unexpected write to input stream."); }

  private:
    static size_t process_response(char const* ptr, size_t size, size_t nmemb, void* data) {
        const size_t written = size * nmemb;

        if (written != 0) {
            auto& buffer = reinterpret_cast<curl_input_stream*>(data)->_buffer;
            buffer.insert(buffer.end(), ptr, ptr + written);
        }
        return written;
    }

    void read() {
        if (_request_completed || _offset < _buffer.size())
            return;

        // Clear the buffer as we've processed its contents
        _buffer.clear();
        _offset = 0;

        read_response();

        if (!_headers_completed)
            check_content_type();
    }

    void check_content_type() {
        long status = 500;
        curl_easy_getinfo(_easy_handle, CURLINFO_RESPONSE_CODE, &status);

        if (status == 0)
            throw runtime_error("failed to connect to server.");

        if (status == 200) {
            char* ct;
            curl_easy_getinfo(_easy_handle, CURLINFO_CONTENT_TYPE, &ct);
            const string content_type{ ct };
            // Ensure the response is JSON
            if (boost::starts_with(content_type, "application/json")) {
                _headers_completed = true;
                return;
            }
        }

        throw error_response_exception(
            status,
            _buffer.empty() ? string{} : string(_buffer.begin(),
                                                _buffer.end()));
    }

    void read_response() {
        if (_request_completed)
            return;

        do {
            if (curl_multi_wait(_multi_handle, nullptr, 0, -1, nullptr) != CURLM_OK)
                throw runtime_error("failed to wait on multi handle.");

            int remaining = 0;
            if (curl_multi_perform(_multi_handle, &remaining) != CURLM_OK)
                throw runtime_error("failed to perform multi action.");
            if (remaining == 0) {
                _request_completed = true;
                break;
            }
        } while (_buffer.empty());
    }

    size_t _offset = 0;
    size_t _read_count = 0;
    vector<Ch> _buffer;
    CURLM* _multi_handle = nullptr;
    CURL* _easy_handle = nullptr;
    struct curl_slist* _headers = NULL;
    bool _headers_completed = false;
    bool _request_completed = false;
};

void
pdb_query(const PuppetDBConn& conn,
          const string& endpoint,
          const string& query_str) {
    const string server_url = conn.getServerUrl() + endpoint;
    const auto ssl_creds = conn.getSSLCredentials();

    try {
        char buffer[65000];
        rapidjson::FileWriteStream output{ stdout, buffer, sizeof(buffer) };
        rapidjson::PrettyWriter<rapidjson::FileWriteStream> writer{ output };

        rapidjson::Reader reader;
        const string post_fields = convert_query_to_post_data(query_str);
        curl_input_stream input{ server_url, post_fields, ssl_creds };
        if (!reader.Parse<rapidjson::kParseValidateEncodingFlag>(input, writer)) {
            logging::colorize(nowide::cerr, logging::log_level::fatal);
            nowide::cerr << "error: response was not valid JSON." << endl;
            logging::colorize(nowide::cerr);
        }
        nowide::cout << endl;
    } catch (error_response_exception const& ex) {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "unexpected response (status " << ex.status() << "):\n"
                     << ex.response() << endl;
        logging::colorize(nowide::cerr);
    }
}

size_t write_data(char *ptr, size_t size, size_t nmemb, FILE *stream) {
    const size_t written = fwrite(ptr, size, nmemb, stream);
    return written;
}

size_t write_body(char *ptr, size_t size, size_t nmemb, void *userdata) {
    return size * nmemb;
}


void
pdb_export(const PuppetDBConn& conn,
           const string& path,
           const string& anonymization) {
    auto curl = unique_ptr< CURL, function<void(CURL*)> >(curl_easy_init(),
                                                          curl_easy_cleanup);

    const auto ssl_creds = conn.getSSLCredentials();
    if (!ssl_creds.cacert.empty()) curl_easy_setopt(curl.get(), CURLOPT_CAINFO, ssl_creds.cacert.c_str());
    if (!ssl_creds.cert.empty()) curl_easy_setopt(curl.get(), CURLOPT_SSLCERT, ssl_creds.cert.c_str());
    if (!ssl_creds.key.empty()) curl_easy_setopt(curl.get(), CURLOPT_SSLKEY, ssl_creds.key.c_str());

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
    auto curl = unique_ptr< CURL, function<void(CURL*)> >(curl_easy_init(),
                                                          curl_easy_cleanup);

    const auto ssl_creds = conn.getSSLCredentials();
    if (!ssl_creds.cacert.empty()) curl_easy_setopt(curl.get(), CURLOPT_CAINFO, ssl_creds.cacert.c_str());
    if (!ssl_creds.cert.empty()) curl_easy_setopt(curl.get(), CURLOPT_SSLCERT, ssl_creds.cert.c_str());
    if (!ssl_creds.key.empty()) curl_easy_setopt(curl.get(), CURLOPT_SSLKEY, ssl_creds.key.c_str());

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
