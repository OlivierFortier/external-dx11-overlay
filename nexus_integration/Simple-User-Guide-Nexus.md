# Nexus Integration (Alternative Loader)

This project also supports integration with the [Nexus Addon Loader & Manager](https://raidcore.gg/Nexus).

## Installation Steps with Nexus Addon Loader & Manager

### Step 1: Install Nexus

First, download and install Nexus Addon Loader & Manager from the [official website](https://raidcore.gg/Nexus). Follow their installation instructions for your platform.

### Step 2: Download the DLL (Nexus Version)

Go to the [release section](https://github.com/OlivierFortier/external-dx11-overlay/releases) of this repository and download the latest release. Inside the zip file, look for the dll file named `external_dx11_overlay_nexus.dll`. This is the version compiled with Nexus support.

### Step 3: Download the right Blish HUD version

There is an already compiled version of Blish HUD that is compatible with both the Nexus version and the regular version. It is inside of the zip file you downloaded earlier. You don't have to download it separately.

### Step 4: Install the Addon

1. Extract the contents of the zip file to your Guild Wars 2 addons directory.
2. Copy the `external_dx11_overlay_nexus.dll` file contained within the extracted folder to your Guild Wars 2 addons directory.
3. Launch Guild Wars 2 with Nexus enabled
4. Once in-game, enable the "External DX11 overlay loader" addon in Nexus. It should appear in the "installed" tab. A new icon will appear next to the Nexus icon, indicating the addon is active.

### Step 5: Running Blish HUD

On Linux, you need to run the downloaded Blish HUD executable with the same wine prefix and binary as Guild Wars 2.

As mentioned in the in-game UI, __since you already use Nexus__, it is recommended to use the [Gw2 Executable Runner](https://github.com/OlivierFortier/gw2-executable-runner) addon to run external programs such as Blish HUD from within the game. This is particularly useful for Linux and SteamOS users in Game Mode. You can use the link provided to download the addon, or grab it from the Nexus addon library.

Once the Gw2 Executable Runner addon is installed, you can configure it to point to the Blish HUD executable you downloaded earlier. This way, you can launch Blish HUD directly from within Guild Wars 2. You can also configure it to launch Blish HUD automatically when you start the game.

On Windows, you can run the Blish HUD executable directly, but the Linux-recommended method also works.
