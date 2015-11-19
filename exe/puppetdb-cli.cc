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
        "  puppetdb-cli [options]\n"
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
            ("log-level,l", po::value<log_level>()->default_value(log_level::warning, "warn"), "Set logging level.\nSupported levels are: none, trace, debug, info, warn, error, and fatal.")
            ("version,v", "print the version and exit");

        po::variables_map vm;

        try {
            po::store(po::parse_command_line(argc, argv, command_line_options), vm);

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
    } catch (exception& ex) {
        colorize(boost::nowide::cerr, log_level::fatal);
        boost::nowide::cerr << "unhandled exception: " << ex.what() << "\n" << endl;
        colorize(boost::nowide::cerr);
        return EXIT_FAILURE;
    }

    boost::nowide::cout << "Hello!" << endl;

    return error_has_been_logged() ? EXIT_FAILURE : EXIT_SUCCESS;
}
