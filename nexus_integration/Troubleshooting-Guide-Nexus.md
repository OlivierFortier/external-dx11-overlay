# Troubleshooting Guide - Nexus Integration

This guide is meant to help debug issues as easily as possible. Common issue can be found towards the end. The two goals are as follows:

- Help you find exactly what your issue is and how to solve it
- Failing that, help you pinpoint the exact cause so it can be reported in a way that's easy to fix.

## What is causing my issue?

The first important step is understanding where the issue lies. There are different components involved in making this work.

1. **Nexus framework** - The addon loader & management system
2. **The external_dx11_overlay_nexus.dll** (Nexus version)
3. **BlishHUD executable**
4. **Nexus addon configuration**

Figuring out which component is causing the issue makes troubleshooting much easier.

### Is Nexus Loading Properly?

- **Check if Nexus is running**: Look for a small stylized "X" icon somewhere in the top-left of your screen.
- **Verify addon is enabled**: In-game, check that "External DX11 overlay loader" is enabled in Nexus under the "installed" tab.
- **Check Nexus logs**: Nexus has its own logging system. In the Nexus menu there is a "logs" option to see all the nexus related logs, including those of all addons loaded by Nexus.
- **Test other addons**: Try enabling/disabling other Nexus addons to verify Nexus itself is working

### Nexus-Specific Issues

#### The addon is not showing up in the "installed" addons tab inside Nexus

- **Wrong DLL version** : Make sure you're using the Nexus-compatible version of this addon.

#### Configuration Window Won't Open

- **Keybind conflict**: ALT+SHIFT+1 (default keybind for opening the nexus blish loader menu) might be bound to something else in GW2 or another addon
- **Addon not loaded**: Verify the DLL was placed in the correct Gw2 addons directory, and that it is **enabled** in the **installed** addons tab.

#### BlishHUD Won't Launch from Nexus Interface

- **Incorrect executable path**: Double-check the path in the configuration window
- **Missing dependencies**: Verify dotnet48 or other requirements are met in your environment
- **Incompatible BlishHUD version:** You cannot use the regular BlishHUD version from the official Blish website or github repo. You need to download a specific version, included in the zip file [in the &#34;releases&#34; of this repository](https://github.com/SorryQuick/external-dx11-overlay/releases). Alternatively you can compile it yourself from [this forked repository](https://github.com/SorryQuick/Blish-HUD).

### Switching Between Methods

If you're having issues with Nexus integration, you can temporarily switch back to the traditional method:

1. Backup your current addons directory to keep your Nexus addons setup safe.
2. Uninstall Nexus.
3. Use the traditional setup process from the [Simple User Guide](../../Simple-User-Guide.md).
4. This can help determine if the issue is with your BlishHUD setup or the Nexus integration specifically

### The DLL is the problem. Now what?

- Any and all panics or crashes as well as regular logs are all found in LOADER_public/logs/dll-xxxxxxx
- They can also be seen in-game in the debug overlay by pressing (by default) CTRL-ALT-D. They are not as detailed as the log file itself.
- If you are experiencing performance issues, you can pinpoint if it's rendering or processing related by disabling either or both of them temporarily, with CTRL-ALT-B and CTRL-ALT-N respectively. Expect visual glitches.
- If the game itself crashes, then the DLL is the problem.
- Generally, this is the part least prone to silent failure. If it fails, it will panic, crash an/or freeze the game, but will generally not fail while the game keeps working flawlessly.

If it did not help, or if you are now left with some sort of an error, report an issue here on github, or ask on the blishhud discord. Make sure to include as much information as you managed to gather.

## Investigating Crashes

The first step is to figure out what part crashes.

- If the game is launched successfully, then it is not the loader. Look for logs in LOADER_public/logs or terminal output. This can be run independently.
- If the game itself crashes, then it is definitely the DLL. Look for logs in LOADER_public/logs.
- The easiest way to tell if Blish itself crashed is to check with top/ps/htop/btop or any task manager and see if it's still running. You can also look for an icon in your system tray. You can look for logs in LOADER_public/logs, but it's entirely possible for it to crash silently. You can try to restart it with CTRL-ALT-O.

From there, open an issue on github or ask for help on the blishhud discord, or nexus discord if relevant to your problem. Make sure to include as much information as you managed to gather.

## Common Issues / Things to verify

**Nexus Integration specific:**

- Ensure you're using the correct DLL and not the standard build.
- Verify Nexus is properly installed and running before launching GW2
- Check that the addon appears in Nexus's addon list (if it doesn't appear, the DLL might be in the wrong location, corrupted or you're using the wrong DLL)
- If the configuration window doesn't open with ALT+SHIFT+1, check for keybind conflicts with GW2 or other addons, or change it in the Nexus settings. Anyway the menu icon should still work even if the keybind doesn't.

## Nexus-specific Known issues

- Unloading the addon while the game is running causes a crash and potentially a memory leak. This is normal as it is not properly implemented yet.

### Windows-specific:
- Doesn't really work with windowed mode. Often creates a black screen with the nexus UI flickering.
- Weird scaling issue on fullscreen-windowed mode. The game and overlay works but sometimes the game can get stretched outside the screen and the event listener areas (where you click) are offset from where the actual UI is.
