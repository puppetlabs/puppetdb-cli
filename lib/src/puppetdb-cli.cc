#include <stdio.h>
#include <curl/curl.h>
#include <string>

#include <boost/nowide/iostream.hpp>
#include <boost/nowide/args.hpp>
#include <boost/filesystem.hpp>

#include <leatherman/logging/logging.hpp>
#include <leatherman/json_container/json_container.hpp>
#include <leatherman/curl/client.hpp>
#include <leatherman/curl/request.hpp>
#include <leatherman/curl/response.hpp>
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
    const string pdbrc_path { futil::tilde_expand("~/.puppetlabs/client-tools/puppetdb.conf") };
    const json::JsonContainer default_config("{\"environments\":{\"dev\":{\"server_urls\":[\"http://127.0.0.1:8080\"]}}}");
    if (fs::exists(pdbrc_path)) {
        const json::JsonContainer raw_config(futil::read(pdbrc_path));
        const auto host = raw_config.getWithDefault<string>("default_environment", "dev");
        return raw_config.getWithDefault<json::JsonContainer>({"environments", host}, default_config);
    } else {
        return default_config;
    }
}

curl::client
pdb_client(const json::JsonContainer& config) {
    auto cacert = config.getWithDefault<string>("cacert", "");
    auto cert = config.getWithDefault<string>("cert", "");
    auto key = config.getWithDefault<string>("key", "");
    curl::client client;
    client.set_ca_cert(cacert);
    client.set_client_cert(cert, key);
    return client;
}

string
pdb_server_url(const json::JsonContainer& config) {
    const auto server_urls = config.get< vector<string> >("server_urls");
    return server_urls.size() ? server_urls[0] : "http://127.0.0.1:8080";
}

curl::request
pdb_query_request(const string& server_url,
                  const json::JsonContainer& query) {
    json::JsonContainer request_body;
    if (!query.empty()) request_body.set("query", query);
    curl::request request(server_url + "/pdb/query/v4");
    request.body(request_body.toString(), "application/json");
    return request;
}

void
pdb_query(const json::JsonContainer& config,
          const json::JsonContainer& query) {
    const auto server_url = pdb_server_url(config);
    const auto response = pdb_client(config).post(pdb_query_request(server_url, query));
    if (response.status_code() >= 200 && response.status_code() < 300) {
        json::JsonContainer response_body(response.body());
        nowide::cout << response_body.toString() << endl;
    } else {
        logging::colorize(nowide::cerr, logging::log_level::error);
        nowide::cerr << "error: " << response.body() << endl;
        logging::colorize(nowide::cerr);
    }
    return;
}

size_t
write_data(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    const size_t written = fwrite(ptr, size, nmemb, stream);
    return written;
}

unique_ptr<CURL, function<void(CURL*)> >
pdb_curl_handler(const json::JsonContainer& config) {
    const auto cacert = config.getWithDefault<string>("cacert", "");
    const auto cert = config.getWithDefault<string>("cert", "");
    const auto key = config.getWithDefault<string>("key", "");

    auto curl = unique_ptr< CURL, function<void(CURL*)> >(curl_easy_init(), curl_easy_cleanup);

    if (cacert != "") curl_easy_setopt(curl.get(), CURLOPT_CAINFO, cacert.c_str());
    if (cert != "") curl_easy_setopt(curl.get(), CURLOPT_SSLCERT, cert.c_str());
    if (key != "") curl_easy_setopt(curl.get(), CURLOPT_SSLKEY, key.c_str());
    return curl;
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

    auto fp = unique_ptr< FILE, function<void(FILE*)> >(fopen(path.c_str(), "wb"), fclose);
    curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, fp.get());
    curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_data);
    curl_easy_perform(curl.get());
}

}  // puppetdb_cli
