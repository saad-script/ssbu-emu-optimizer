# SSBU Emulator Optimizer

An application to easily setup Super Smash Bros Ultimate on your emulator with optimal settings and mods for competetive play.
This app requires Microsoft Webview2 which should be installed on windows by default, but if the app detects that it isn't installed it will prompt you to install it.

### Prerequisites

- Installed emulator
- Added keys to emulator
- Added SSBU game directory to emulator
- Installed latest ssbu update (13.0.4) on emulator

### Usage

- OPTIONAL: Select your emulator data folder in the top left.
- OPTIONAL: Select which emulator user profile you want to optimize in the top left.
- Select 1 or more of the 3 optimization options (settings, mods, save)
- Click "Optimize Selected"
- Thats it! You can open emulator and launch SSBU

### Options

- Settings
  - This option will load in the optimal settings for SSBU in your emulator. This will load the options in the game properties so these settings will only affect SSBU and no other games.
- Save
  - This will load in a 100% SSBU save with all characters unlocked and also predefined rulesets for competitive play. 
- Mods
  - This will add Atmosphere, Skyline, Arcropolis and all files required for modding and also several mods to increase performance and quality of life:
  - Hollow Bastion with the Eternal Heart mod (use this mod to maximize performance on Hollow Bastion)
  - CSS Preserve (to keep the same character after a set in LDN mode/offline)
  - One Slot Effects (This allows any effect mod to be used on multiple slots simultaneously, meaning they are not fake one-slot)
  - Training Mod Pack
  - The SSBU Online Deluxe Mod
    - Allows you to reduce the added online latency
    - Allows you to apply render profiles to reduce the native input delay of the game.
