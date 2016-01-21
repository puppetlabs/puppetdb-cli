#include <stdio.h>
#include <curl/curl.h>
#include <string>

#include <boost/nowide/iostream.hpp>
#include <boost/nowide/args.hpp>
#include <boost/filesystem.hpp>

#include <leatherman/logging/logging.hpp>
#include <leatherman/json_container/json_container.hpp>
#include <leatherman/file_util/file.hpp>

#include <puppetdb-cli/version.h>
#include <puppetdb-cli/puppetdb-cli.hpp>

namespace puppetdb_cli {

using namespace std;
namespace fs = boost::filesystem;
namespace nowide = boost::nowide;
namespace json = leatherman::json_container;
namespace futil = leatherman::file_util;
namespace logging = leatherman::logging;
namespace curl = leatherman::curl;

string
version()
{
    LOG_DEBUG("puppetdb-cli version is %1%", PUPPETDB_CLI_VERSION_WITH_COMMIT);
    return PUPPETDB_CLI_VERSION_WITH_COMMIT;
}

json::JsonContainer
parse_config() {
    const string pdbrc_path
    { futil::tilde_expand("~/.puppetlabs/client-tools/puppetdb.conf") };
    const string default_config_str
    { "{\"environments\":{\"dev\":{\"server_urls\":[\"http://127.0.0.1:8080\"]}}}" };
    const json::JsonContainer default_config(default_config_str);
    if (fs::exists(pdbrc_path)) {
        const json::JsonContainer raw_config(futil::read(pdbrc_path));
        const auto host = raw_config
                .getWithDefault<string>("default_environment", "dev");
        return raw_config
                .getWithDefault<json::JsonContainer>({"environments", host},
                                                     default_config);
    } else {
        return default_config;
    }
}

string
pdb_server_url(const json::JsonContainer& config) {
    const auto server_urls = config.get< vector<string> >("server_urls");
    return server_urls.size() ? server_urls[0] : "http://127.0.0.1:8080";
}

size_t
write_data(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    const size_t written = fwrite(ptr, size, nmemb, stream);
    return written;
}

size_t write_body(char *ptr, size_t size, size_t nmemb, void *userdata){
    return size * nmemb;
}

unique_ptr<CURL, function<void(CURL*)> >
pdb_curl_handler(const json::JsonContainer& config) {
    const auto cacert = config.getWithDefault<string>("cacert", "");
    const auto cert = config.getWithDefault<string>("cert", "");
    const auto key = config.getWithDefault<string>("key", "");

    auto curl = unique_ptr< CURL, function<void(CURL*)> >(curl_easy_init(),
                                                          curl_easy_cleanup);

    if (cacert != "") curl_easy_setopt(curl.get(), CURLOPT_CAINFO, cacert.c_str());
    if (cert != "") curl_easy_setopt(curl.get(), CURLOPT_SSLCERT, cert.c_str());
    if (key != "") curl_easy_setopt(curl.get(), CURLOPT_SSLKEY, key.c_str());
    return curl;
}

void
pdb_query(const json::JsonContainer& config,
          const string& query) {
    auto curl = pdb_curl_handler(config);

    auto url_encoded_query = unique_ptr< char, function<void(char*)> >(
        curl_easy_escape(curl.get(), query.c_str(), query.length()),
        curl_free);

    const auto server_url = pdb_server_url(config) +
            "/pdb/query/v4" +
            "?query=" +
            url_encoded_query.get();

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
pdb_export(const json::JsonContainer& config,
           const string& path,
           const string& anonymization) {
    auto curl = pdb_curl_handler(config);
    const string server_url = pdb_server_url(config)
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

void pdb_import(const json::JsonContainer& config,
                const string& infile,
                const string& command_versions) {
    auto curl = pdb_curl_handler(config);
    curl_httppost* formpost = NULL;
    curl_httppost* lastptr = NULL;

    const string server_url = pdb_server_url(config) + "/pdb/admin/v1/archive";

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

}  // puppetdb_cli
