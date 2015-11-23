#include <puppetdb-cli/puppetdb-cli.hpp>
#include <boost/nowide/iostream.hpp>
#include <boost/nowide/args.hpp>
#include <leatherman/logging/logging.hpp>

// boost includes are not always warning-clean. Disable warnings that
// cause problems before including the headers, then re-enable the warnings.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wattributes"
#include <boost/program_options.hpp>
#pragma GCC diagnostic pop

using namespace std;
using namespace leatherman::logging;
namespace po = boost::program_options;

void help(po::options_description& desc)
{
    boost::nowide::cout <<
        "Synopsis\n"
        "========\n"
        "\n"
        "Example command-line utility.\n"
        "\n"
        "Usage\n"
        "=====\n"
        "\n"
        "  puppetdb-cli [options] context query [paging-options]\n"
        "\n"
        "Options\n"
        "=======\n\n" << desc <<
        "\nDescription\n"
        "===========\n"
        "\n"
        "Displays its own version string." << endl;
}

int main(int argc, char **argv) {
    try {
        // Fix args on Windows to be UTF-8
        boost::nowide::args arg_utf8(argc, argv);

        // Setup logging
        setup_logging(boost::nowide::cerr);

        po::options_description command_line_options("");
        command_line_options.add_options()
            ("help,h", "produce help message")
            ("log-level,l", po::value<log_level>()->default_value(log_level::warning, "warn"),
             "Set logging level.\nSupported levels are: none, trace, debug, info, warn, error, and fatal.")
            ("version,v", "print the version and exit")
            ("context", po::value<string>()->default_value("nodes"), "endpoint for PuppetDB")
            ("query", po::value<string>()->default_value(""), "query for PuppetDB")
            ("limit", po::value<int>()->default_value(-1), "limit paging option for PuppetDB");

        po::positional_options_description positional_options;
        positional_options.add("context", 1).
          add("query", 1);

        po::variables_map vm;

        try {
            po::store(po::command_line_parser(argc, argv).
                      options(command_line_options).
                      positional(positional_options).run(),
                      vm); // throws on error

            if (vm.count("help")) {
                help(command_line_options);
                return EXIT_SUCCESS;
            }

            po::notify(vm);
        } catch (exception& ex) {
            colorize(boost::nowide::cerr, log_level::error);
            boost::nowide::cerr << "error: " << ex.what() << "\n" << endl;
            colorize(boost::nowide::cerr);
            help(command_line_options);
            return EXIT_FAILURE;
        }

        // Get the logging level
        auto lvl = vm["log-level"].as<log_level>();
        set_level(lvl);

        if (vm.count("version")) {
            boost::nowide::cout << puppetdb_cli::version() << endl;
            return EXIT_SUCCESS;
        }

        auto endpoint = vm["context"].as<string>();
        auto query = vm["query"].as<string>();
        auto limit = vm["limit"].as<int>();
        boost::nowide::cout << puppetdb_cli::query(puppetdb_cli::parse_config(),
                                                   endpoint,
                                                   query,
                                                   limit) << endl;
    } catch (exception& ex) {
        colorize(boost::nowide::cerr, log_level::fatal);
        boost::nowide::cerr << "unhandled exception: " << ex.what() << "\n" << endl;
        colorize(boost::nowide::cerr);
        return EXIT_FAILURE;
    }

    return error_has_been_logged() ? EXIT_FAILURE : EXIT_SUCCESS;
}
