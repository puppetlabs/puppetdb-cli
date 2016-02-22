#include <stdio.h>
#include <curl/curl.h>
#include <string>
#include <sstream>
#include <vector>

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
version()
{
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

unique_ptr<CURL, function<void(CURL*)> >
PuppetDBConn::getCurlHandle() const {
    auto curl = unique_ptr< CURL, function<void(CURL*)> >(curl_easy_init(),
                                                          curl_easy_cleanup);
    if (cacert_ != "") curl_easy_setopt(curl.get(), CURLOPT_CAINFO, cacert_.c_str());
    if (cert_ != "") curl_easy_setopt(curl.get(), CURLOPT_SSLCERT, cert_.c_str());
    if (key_ != "") curl_easy_setopt(curl.get(), CURLOPT_SSLKEY, key_.c_str());
    return curl;
}

size_t write_data(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    const size_t written = fwrite(ptr, size, nmemb, stream);
    return written;
}

size_t write_body(char *ptr, size_t size, size_t nmemb, void *userdata){
    return size * nmemb;
}

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

    curl_easy_setopt(curl.get(), CURLOPT_URL, server_url.c_str());
    curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, stdout);
    curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_data);

    const CURLcode curl_code = curl_easy_perform(curl.get());
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
