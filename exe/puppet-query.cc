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
#include <puppetdb-cli/puppetdb-cli.hpp>

using namespace std;
namespace nowide = boost::nowide;
namespace po = boost::program_options;
namespace logging = leatherman::logging;

void
help(po::options_description& global_desc)
{
    nowide::cout << "usage: puppet-query [global] query <query>\n\n"
                 << global_desc << endl;
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

        po::options_description command_line_options("query subcommand options");
        command_line_options.add(global_options).add_options()
                ("query", po::value<string>()->default_value(""),
                 "query to execute against PuppetDB");

        po::positional_options_description positional_options;
        positional_options.add("query", 1);

        po::variables_map vm;

        try {
            po::parsed_options parsed = po::command_line_parser(argc, argv).
                    options(command_line_options).
                    positional(positional_options).
                    run();  // throws on error
            po::store(parsed, vm);

            if (vm.count("help")) {
                help(global_options);
                return EXIT_SUCCESS;
            }

            if (vm.count("version")) {
                nowide::cout << puppetdb_cli::version() << endl;
                return EXIT_SUCCESS;
            }

            po::notify(vm);
        } catch (exception& ex) {
            logging::colorize(nowide::cerr, logging::log_level::error);
            nowide::cerr << "error: " << ex.what() << "\n" << endl;
            logging::colorize(nowide::cerr);
            help(global_options);
            return EXIT_FAILURE;
        }

        // Get the logging level
        const auto lvl = vm["log-level"].as<logging::log_level>();
        logging::set_level(lvl);

        const auto config = puppetdb_cli::parse_config();
        const auto query = vm["query"].as<string>();
        puppetdb_cli::pdb_query(config, query);
    } catch (exception& ex) {
        logging::colorize(nowide::cerr, logging::log_level::fatal);
        nowide::cerr << "unhandled exception: " << ex.what() << "\n" << endl;
        logging::colorize(nowide::cerr);
        return EXIT_FAILURE;
    }

    return logging::error_has_been_logged() ? EXIT_FAILURE : EXIT_SUCCESS;
}
