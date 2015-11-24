#include <puppetdb-cli/version.h>
#include <puppetdb-cli/puppetdb-cli.hpp>

#include <leatherman/logging/logging.hpp>
#include <leatherman/json_container/json_container.hpp>
#include <leatherman/curl/client.hpp>
#include <leatherman/curl/request.hpp>
#include <leatherman/curl/response.hpp>
#include <leatherman/file_util/file.hpp>

#include <boost/filesystem.hpp>

#include <iostream>
namespace puppetdb_cli {

    using namespace std;
    using leatherman::json_container::JsonContainer;
    namespace file_util = leatherman::file_util;
    namespace curl = leatherman::curl;

    string version()
    {
        LOG_DEBUG("puppetdb-cli version is %1%", PUPPETDB_CLI_VERSION_WITH_COMMIT);
        return PUPPETDB_CLI_VERSION_WITH_COMMIT;
    }

    JsonContainer parse_config() {
      string pdbrc_path { file_util::get_home_path() + "/.pdbrc" };
      JsonContainer default_config("{\"environments\":{\"dev\":{\"server_urls\":\"http://127.0.0.1:8080\"}}}");
      if (boost::filesystem::exists(pdbrc_path)) {
        JsonContainer raw_config(file_util::read(pdbrc_path));
        auto host = raw_config.getWithDefault<string>("default_environment", "dev");
        return raw_config.getWithDefault<JsonContainer>({"environments", host}, default_config);
      } else {
        return default_config;
      }
    }

    curl::response query(const JsonContainer& config,
                         const string& endpoint,
                         const string& query_string,
                         const int limit,
                         const string& order_by_string) {
      curl::client client;
      auto cacert = config.getWithDefault<string>("cacert", "");
      auto cert = config.getWithDefault<string>("cert", "");
      auto key = config.getWithDefault<string>("key", "");
      client.set_ca_cert(cacert);
      client.set_client_cert(cert, key);

      auto root_url = config.getWithDefault<string>("server_urls", "http://127.0.0.1:8080");

      JsonContainer request_body;
      if (!query_string.empty()) {
        JsonContainer query_json(query_string);
        request_body.set("query", query_json);
      }
      if (limit > 0) {
        request_body.set("limit", limit);
      }
      if (!order_by_string.empty()) {
        JsonContainer order_by_json(order_by_string);
        request_body.set("order_by", order_by_json);
      }

      curl::request request(root_url + "/pdb/query/v4/" + endpoint);
      request.body(request_body.toString(), "application/json");

      return client.post(request);
    }

}  // puppetdb_cli
