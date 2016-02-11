#include <stdio.h>
#include <curl/curl.h>
#include <string>

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
    const auto server_url = conn.getServerUrl() + "/pdb/query/v4";
    curl_easy_setopt(curl.get(), CURLOPT_URL, server_url.c_str());

    // If this is PQL then we need to wrap the query in double-quotes, otherwise
    // the query is AST and we leave it alone
    string query_str_copy { query_str };
    boost::trim(query_str_copy);
    const string query = (query_str_copy[0] == '[') ? query_str:"\""+ query_str +"\"";
    const string post_data = "{\"query\":" + query + "}";
    curl_easy_setopt(curl.get(), CURLOPT_POSTFIELDS, post_data.c_str());

    auto headers = unique_ptr<curl_slist, function<void(curl_slist*)> >(NULL,
                                                                        curl_slist_free_all);
    curl_easy_setopt(curl.get(),
                     CURLOPT_HTTPHEADER,
                     curl_slist_append(headers.get(), "Content-Type: application/json"));


    curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, stdout);
    curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_data);

    const CURLcode curl_code = curl_easy_perform(curl.get());
    if (curl_code != CURLE_OK) {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "error connecting to PuppetDB: "
                     << curl_easy_strerror(curl_code) << endl;
        logging::colorize(nowide::cerr);
    } else {
        nowide::cout << endl;
        long http_code = 0;
        curl_easy_getinfo(curl.get(), CURLINFO_RESPONSE_CODE, &http_code);
        if (http_code != 200) {
            logging::colorize(nowide::cerr, logging::log_level::fatal);
            nowide::cerr << "error status " << http_code << " contacting PuppetDB" << endl;
            logging::colorize(nowide::cerr);
        }
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
