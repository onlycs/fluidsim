# fluidsim

## Online Demo

Note that this project was never supposed to run in the browser. I made the bare minimum possible changes to get it working with WebGPU. Your computer WILL freeze if it does not have enough RAM or CPU.

That being said, visit [fluidsim.angad.page](https://fluidsim.angad.page) to try it out.

## Windows

Despite my best efforts, this application crashes on my Windows VM. I have only ever gotten it to work on Linux.

That being said, a release binary is available from the [releases page](https://github.com/onlycs/fluidsim/releases). It dynamically links to

- `libvulkan.so.1`
- `libwayland-client.so.0`
- `libxcbcommon.so.0`

So make sure they are available.

## Acknowledgements

- Sebastian Lague for the [YouTube video](https://www.youtube.com/watch?v=rSKMYc1CQHE) that made me think this was a good project idea
- [These](https://matthias-research.github.io/pages/publications/sca03.pdf) [three](https://web.archive.org/web/20250106201614/http://www.ligum.umontreal.ca/Clavet-2005-PVFS/pvfs.pdf) [papers](https://sph-tutorial.physics-simulation.org/pdf/SPH_Tutorial.pdf) from his works cited page
- [These files](https://github.com/SebLague/Fluid-Sim/tree/Episode-01/Assets/Scripts/Sim%202D/Compute) which I used for reference, occasionally.
