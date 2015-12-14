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
namespace nowide = boost::nowide;
namespace po = boost::program_options;
namespace logging = leatherman::logging;
namespace json = leatherman::json_container;

void
help(po::options_description& global_desc,
     po::options_description& export_subcommand_desc,
     po::options_description& import_subcommand_desc)
{
    nowide::cout << "usage: puppet-db [global] export [options]\n"
                 << "       puppet-db [global] import [options]\n"
                 << "       puppet-db [global] query <query>\n\n"
                 << global_desc << "\n"
                 << export_subcommand_desc << "\n"
                 << import_subcommand_desc << endl;
}

int
main(int argc, char **argv) {
    try {
        // Fix args on Windows to be UTF-8
        nowide::args arg_utf8(argc, argv);

        // Setup logging
        logging::setup_logging(nowide::cerr);

        po::options_description global_options("global options");
        global_options.add_options()
                ("help,h", "produce help message")
                ("log-level,l",
                 po::value<logging::log_level>()->default_value(logging::log_level::warning,
                                                                "warn"),
                 "Set logging level.\n"
                 "Supported levels are: none, trace, debug, info, warn, error, and fatal.")
                ("version,v", "print the version and exit");

        po::options_description command_line_options("");
        command_line_options.add(global_options).add_options()
                ("subcommand", po::value<string>(),
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
                ("outfile", po::value<string>()->default_value("./pdb-export.tgz"),
                 "path to create PuppetDB archive")
                ("anonymization", po::value<string>()->default_value("none"),
                 "anonymization for the PuppetDB archive");

        po::options_description import_subcommand_options("import subcommand options");
        import_subcommand_options.add_options()
                ("infile", po::value<string>(),
                 "the file to import into PuppetDB")
                ("command-versions", po::value<string>(),
                 "command versions to use for import");

        po::variables_map vm;

        try {
            po::parsed_options parsed = po::command_line_parser(argc, argv).
                    options(command_line_options).
                    positional(positional_options).
                    allow_unregistered().
                    run();  // throws on error

            po::store(parsed, vm);

            if (vm.count("help") || vm["subcommand"].empty()) {
                help(global_options,
                     export_subcommand_options,
                     import_subcommand_options);
                return EXIT_SUCCESS;
            }

            if (vm.count("version")) {
                nowide::cout << puppetdb_cli::version() << endl;
                return EXIT_SUCCESS;
            }

            const auto subcommand = vm["subcommand"].as<string>();
            if ((!(subcommand == "query") &&
                 !(subcommand == "export") &&
                 !(subcommand == "import"))
                || (((subcommand == "query") || (subcommand == "import"))
                    && vm["subargs"].empty())) {
                help(global_options,
                     export_subcommand_options,
                     import_subcommand_options);
                return EXIT_FAILURE;
            }

            // Collect all the unrecognized options from the first pass. This will include the
            // (positional) command name, so we need to erase that.
            vector<string> opts = po::collect_unrecognized(parsed.options,
                                                           po::include_positional);
            opts.erase(opts.begin());
            if (subcommand == "query"){
                po::store(po::command_line_parser(opts).options(query_subcommand_options)
                          .positional(query_positional_options)
                          .run(), vm);
            } else if (subcommand == "export") {
                po::store(po::command_line_parser(opts).options(export_subcommand_options)
                          .run(), vm);
            } else if (subcommand == "import") {
                po::store(po::command_line_parser(opts).options(import_subcommand_options)
                          .run(), vm);
            }

            po::notify(vm);
        } catch (exception& ex) {
            logging::colorize(nowide::cerr, logging::log_level::error);
            nowide::cerr << "error: " << ex.what() << "\n" << endl;
            logging::colorize(nowide::cerr);
            help(global_options,
                 export_subcommand_options,
                 import_subcommand_options);
            return EXIT_FAILURE;
        }

        // Get the logging level
        const auto lvl = vm["log-level"].as<logging::log_level>();
        logging::set_level(lvl);

        const auto subcommand = vm["subcommand"].as<string>();
        const auto config = puppetdb_cli::parse_config();
        if (subcommand == "query") {
            const json::JsonContainer query{ vm["query"].as<string>() };
            puppetdb_cli::pdb_query(config, query);
        } else if (subcommand == "export") {
            puppetdb_cli::pdb_export(config,
                                     vm["outfile"].as<string>(),
                                     vm["anonymization"].as<string>());
        } else if (subcommand == "import") {
            puppetdb_cli::pdb_import(config,
                                     vm["infile"].as<string>(),
                                     vm["command-versions"].as<string>());
        }
    } catch (exception& ex) {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "unhandled exception: " << ex.what() << "\n" << endl;
        logging::colorize(nowide::cerr);
        return EXIT_FAILURE;
    }

    return logging::error_has_been_logged() ? EXIT_FAILURE : EXIT_SUCCESS;
}
