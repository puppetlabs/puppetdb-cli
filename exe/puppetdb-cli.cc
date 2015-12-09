// boost includes are not always warning-clean. Disable warnings that
// cause problems before including the headers, then re-enable the warnings.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wattributes"
#include <boost/nowide/iostream.hpp>
#include <boost/nowide/args.hpp>
#include <boost/program_options.hpp>
#include <boost/filesystem.hpp>
#include <boost/filesystem/fstream.hpp>
#pragma GCC diagnostic pop
#include <leatherman/logging/logging.hpp>
#include <leatherman/curl/response.hpp>
#include <leatherman/json_container/json_container.hpp>
#include <puppetdb-cli/puppetdb-cli.hpp>

using namespace std;
namespace po = boost::program_options;
namespace filesystem = boost::filesystem;
using namespace leatherman::logging;
namespace curl = leatherman::curl;
using leatherman::json_container::JsonContainer;

void
help(po::options_description& global_desc,
     po::options_description& export_subcommand_desc)
{
    boost::nowide::cout <<
            "usage: puppet-db [global] export [options]\n" <<
            "       puppet-db [global] query <query>\n\n" <<
            global_desc << "\n" <<
            export_subcommand_desc;
}

int
main(int argc, char **argv) {
    try {
        // Fix args on Windows to be UTF-8
        boost::nowide::args arg_utf8(argc, argv);

        // Setup logging
        setup_logging(boost::nowide::cerr);

        po::options_description global_options("global options");
        global_options.add_options()
                ("help,h", "produce help message")
                ("log-level,l", po::value<log_level>()->default_value(log_level::warning, "warn"),
                 "Set logging level.\n"
                 "Supported levels are: none, trace, debug, info, warn, error, and fatal.")
                ("version,v", "print the version and exit");

        po::options_description command_line_options("");
        command_line_options.add(global_options).add_options()
                ("subcommand", po::value<string>()->default_value(""),
                 "subcommand to execute")
                ("subargs", po::value< vector<string> >(),
                 "arguments for subcommand");

        po::positional_options_description positional_options;
        positional_options.add("subcommand", 1).add("subargs", -1);

        po::options_description query_subcommand_options("query subcommand options");
        query_subcommand_options.add_options()
                ("query", po::value<string>()->default_value(""),
                 "query PuppetDB data");

        po::positional_options_description query_positional_options;
        query_positional_options.add("query", 1);

        po::options_description export_subcommand_options("export subcommand options");
        export_subcommand_options.add_options()
                ("path", po::value<string>()->default_value("./pdb-export.tgz"),
                 "path to create PuppetDB archive")
                ("anonymization", po::value<string>()->default_value("none"),
                 "anonymization for the PuppetDB archive");

        po::variables_map vm;

        try {
            po::parsed_options parsed = po::command_line_parser(argc, argv).
                    options(command_line_options).
                    positional(positional_options).
                    allow_unregistered().
                    run();  // throws on error

            po::store(parsed, vm);

            if (vm.count("help")) {
                help(global_options, export_subcommand_options);
                return EXIT_SUCCESS;
            }

            if (vm.count("version")) {
                boost::nowide::cout << puppetdb_cli::version() << endl;
                return EXIT_SUCCESS;
            }

            auto subcommand = vm["subcommand"].as<string>();
            if ((!(subcommand == "query") && !(subcommand == "export"))
                || ((subcommand == "query") && vm["subargs"].as< vector<string> >().empty())) {
                help(global_options, export_subcommand_options);
                return EXIT_SUCCESS;
            }

            // Collect all the unrecognized options from the first pass. This will include the
            // (positional) command name, so we need to erase that.
            vector<string> opts = po::collect_unrecognized(parsed.options,
                                                           po::include_positional);
            opts.erase(opts.begin());
            if (subcommand == "query") {
                po::store(po::command_line_parser(opts).options(query_subcommand_options)
                          .positional(query_positional_options)
                          .run(), vm);
            }
            if (subcommand == "export") {
                po::store(po::command_line_parser(opts).options(export_subcommand_options)
                          .run(), vm);
            }


            po::notify(vm);
        } catch (exception& ex) {
            colorize(boost::nowide::cerr, log_level::error);
            boost::nowide::cerr << "error: " << ex.what() << "\n" << endl;
            colorize(boost::nowide::cerr);
            help(global_options, export_subcommand_options);
            return EXIT_FAILURE;
        }

        // Get the logging level
        auto lvl = vm["log-level"].as<log_level>();
        set_level(lvl);

        auto subcommand = vm["subcommand"].as<string>();
        auto config = puppetdb_cli::parse_config();
        if (subcommand == "query") {
            JsonContainer query{ vm["query"].as<string>() };

            auto response = puppetdb_cli::pdb_query(config, query);
            if (response.status_code() >= 200 && response.status_code() < 300) {
                JsonContainer response_body(response.body());
                boost::nowide::cout << response_body.toString() << endl;
            } else {
                colorize(boost::nowide::cerr, log_level::error);
                boost::nowide::cerr << "error: " << response.body() << endl;
                colorize(boost::nowide::cerr);
            }
        }
        if (subcommand ==  "export") {
            auto anonymization = vm["anonymization"].as<string>();
            filesystem::path path{ vm["path"].as<string>() };
            filesystem::ofstream ofs{ path };
            boost::nowide::cout << "Exporting PuppetDB..." << endl;
            auto response = puppetdb_cli::pdb_export(config, anonymization);
            ofs << response.body() << endl;
            boost::nowide::cout << "Finished exporting PuppetDB archive to " << path << "." << endl;
        }
    } catch (exception& ex) {
        colorize(boost::nowide::cerr, log_level::fatal);
        boost::nowide::cerr << "unhandled exception: " << ex.what() << "\n" << endl;
        colorize(boost::nowide::cerr);
        return EXIT_FAILURE;
    }

    return error_has_been_logged() ? EXIT_FAILURE : EXIT_SUCCESS;
}
