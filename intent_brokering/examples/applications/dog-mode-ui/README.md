# Dog Mode UI

This solution contains a web front-end served by a ASP.NET back-end
application that implements a user interface and provides a demonstration of
the SDV Application Programming Model through the dog mode scenario.

## Setup

You will need the .NET SDK and ASP.NET Core Runtime version 6. As of the writing of this,
installing the .NET SDK on Ubuntu installs the SDK, runtime, and ASP.NET Core runtime.

If you do not have these already, follow the instructions
[here](https://learn.microsoft.com/en-us/dotnet/core/install/linux-ubuntu-2004#add-the-microsoft-package-repository),
but replace the current version of the SDK with version 6 (dotnet-sdk-6.0).

Once the update is done, run:

```bash
dotnet --info
```

to ensure the installation was successful. At the end of the output message, you should see
something like the following. Ensure that they are major version 6, and that you have both the
SDK and ASP.NET Core runtime.

```bash
.NET SDKs installed:
  6.0.412 [/usr/share/dotnet/sdk]

.NET runtimes installed:
  Microsoft.AspNetCore.App 6.0.20 [/usr/share/dotnet/shared/Microsoft.AspNetCore.App]
  Microsoft.NETCore.App 6.0.20 [/usr/share/dotnet/shared/Microsoft.NETCore.App]
```

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

    ./intent_brokering/examples/applications/dog-mode-ui/mock_provider_dog_mode_demo.sh | cargo run ...
