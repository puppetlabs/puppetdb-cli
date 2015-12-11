#include <stdio.h>
#include <curl/curl.h>
#include <string>

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
    using leatherman::json_container::JsonContainer;
    namespace file_util = leatherman::file_util;
    namespace curl = leatherman::curl;

    string
    version()
    {
        LOG_DEBUG("puppetdb-cli version is %1%", PUPPETDB_CLI_VERSION_WITH_COMMIT);
        return PUPPETDB_CLI_VERSION_WITH_COMMIT;
    }

    JsonContainer
    parse_config() {
      string pdbrc_path { file_util::tilde_expand("~/.puppetlabs/client-tools/puppetdb.conf") };
      JsonContainer default_config("{\"environments\":{\"dev\":{\"server_urls\":[\"http://127.0.0.1:8080\"]}}}");
      if (boost::filesystem::exists(pdbrc_path)) {
        JsonContainer raw_config(file_util::read(pdbrc_path));
        auto host = raw_config.getWithDefault<string>("default_environment", "dev");
        return raw_config.getWithDefault<JsonContainer>({"environments", host}, default_config);
      } else {
        return default_config;
      }
    }

    curl::client
    pdb_client(const JsonContainer& config) {
      curl::client client;
      auto cacert = config.getWithDefault<string>("cacert", "");
      auto cert = config.getWithDefault<string>("cert", "");
      auto key = config.getWithDefault<string>("key", "");
      client.set_ca_cert(cacert);
      client.set_client_cert(cert, key);
      return client;
    }

    curl::response
    pdb_query(const JsonContainer& config,
              const JsonContainer& query) {
      curl::client client{ pdb_client(config) };

      auto server_urls = config.get< vector<string> >("server_urls");
      auto server_url = (server_urls.size() > 0) ? server_urls[0] : "http://127.0.0.1:8080";

      JsonContainer request_body;
      if (!query.empty()) request_body.set("query", query);
      curl::request request(server_url + "/pdb/query/v4");
      request.body(request_body.toString(), "application/json");

      return client.post(request);
    }

    size_t
    write_data(void *ptr, size_t size, size_t nmemb, FILE *stream) {
        size_t written = fwrite(ptr, size, nmemb, stream);
        return written;
    }

    void
    pdb_export(const JsonContainer& config,
               const string& path,
               const string& anonymization) {
        auto cacert = config.getWithDefault<string>("cacert", "");
        auto cert = config.getWithDefault<string>("cert", "");
        auto key = config.getWithDefault<string>("key", "");

        auto curl = shared_ptr<CURL>(curl_easy_init(), curl_easy_cleanup);

        if (cacert != "") curl_easy_setopt(curl.get(), CURLOPT_CAINFO, cacert.c_str());
        if (cert != "") curl_easy_setopt(curl.get(), CURLOPT_SSLCERT, cert.c_str());
        if (key != "") curl_easy_setopt(curl.get(), CURLOPT_SSLKEY, key.c_str());

        auto server_urls = config.get< vector<string> >("server_urls");
        auto server_url = (server_urls.size() > 0) ? server_urls[0] : "http://127.0.0.1:8080";

        string url = "http://localhost:8080/pdb/admin/v1/archive?anonymization=" + anonymization;

        auto fp = shared_ptr<FILE>(fopen(path.c_str(), "wb"), fclose);
        curl_easy_setopt(curl.get(), CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_data);
        curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, fp.get());
        curl_easy_perform(curl.get());
    }

}  // puppetdb_cli
