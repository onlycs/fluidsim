# fluidsim

## Running

### Windows

You may need to install the [redist](https://aka.ms/vs/17/release/vc_redist.x64.exe) first.

If you've ever played a 3d video game or coded in a not-interpreted language, this is probably already on your computer.

After that, grab a release binary from the [releases page](https://github.com/onlycs/fluidsim/releases).

### MacOS

Download and extract the provided zip from the [releases page](https://github.com/onlycs/fluidsim/releases). Open a terminal and run

```bash
xattr -cr /path/to/FluidSim.app
```

to remove the quarantine attribute. You should then be able to double-click on it to run it.

### Linux

A release binary is available from the [releases page](https://github.com/onlycs/fluidsim/releases). Make sure the following are available somewhere

- `glibc` 2.39 or later
- `libwayland` and friends
- `libvulkan` and Mesa ICDs
- `libxcb` and friends
- `libxkbcommon`

## Acknowledgements

- Sebastian Lague for the [YouTube video](https://www.youtube.com/watch?v=rSKMYc1CQHE) that made me think this was a good project idea
- [These](https://matthias-research.github.io/pages/publications/sca03.pdf) [three](https://web.archive.org/web/20250106201614/http://www.ligum.umontreal.ca/Clavet-2005-PVFS/pvfs.pdf) [papers](https://sph-tutorial.physics-simulation.org/pdf/SPH_Tutorial.pdf) from his works cited page
- [These files](https://github.com/SebLague/Fluid-Sim/tree/Episode-01/Assets/Scripts/Sim%202D/Compute) which I used for reference, occasionally.
