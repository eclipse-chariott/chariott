# Dog Mode UI

This solution contains a web front-end served by a ASP.NET back-end
application that implements a user interface and provides a demonstration of
the SDV Application Programming Model through the dog mode scenario.

## Setup

The instructions in this section assume the current working directory of the
shell is the same directory where this document.

Run the following script to install the required .NET SDK:

    ./install.sh

When the script ends successfully, it will print how to update the `PATH`
variable so that the .NET CLI (`dotnet`) can be found at its installed
location. Once the update is done, run:

    dotnet --info

to ensure the installation was successful.

## Running

Execute the following command (assuming the current working directory is the
same as the directory of this document):

     dotnet run --project src

to start the ASP.NET web application. If a browser does not open automtically
at the address of the application, open a browser manually and navigate to
<http://localhost:5079/>.

Other components such as the VAS (Vehicle Abstraction Service), the mock provider and the dog mode
logic application may be started after launching the ASP.NET application.

Use the `mock_provider_dog_mode_demo.sh` script to pipe its output into the mock
VAS to generate mocked sensor data (assuming the current working directory is
the root of the repo):

    ./examples/applications/dog-mode-ui/mock_provider_dog_mode_demo.sh | cargo run ...
