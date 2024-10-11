Macroquad tamogotchi little game
===

You will raise a birds in this game. Probably will add something like motivation tracking (like habitica) or similar kind of stuff.
I try to use lottie animations in there. I wanted to use lottie-rs but there are too much dependencies. I hope I can make my own renderer not complex and able to render stuff that I animate in Synfig/Glaxnimate

# TODO:
- fix metal shaders to do same things as glsl's
- fix metal rendering by providing index_buffer
- fix metall MSAA
- nanoserde: use default instead of None when skip fields

# Resources:
## Lottie:
- [lottie-rs](https://github.com/zimond/lottie-rs) - Ported module structure to nanoserde
## Path rendering:
- [contrast_renderer](https://github.com/Lichtso/contrast_renderer) - Stole most of it for path_rendering module
- [How to draw a complex shape using a stencil buffer](http://web.archive.org/web/20240118160026/https://www.glprogramming.com/red/chapter14.html#name13)
## Path anti-aliasing:
- [Svg to shadertoy generator](https://gist.github.com/Ninja-Koala/74fa7652fb4de248949ce1e27b989c14)
- https://github.com/evanw/theta - currently stealing anti-aliasing from it
- https://github.com/behdad/glyphy - didn't like the result but interesting resource
- https://github.com/jinleili/sdf-text-view - of course cannot compile because of error in wgpu-core
- http://wdobbie.com/post/gpu-text-rendering-with-vector-textures/
- https://medium.com/@evanwallace/easy-scalable-text-rendering-on-the-gpu-c3f4d782c5ac
- https://handmade.network/forums/t/8799-anti-aliasing_in_fragment_shader
- https://www.shadertoy.com/view/DtXcRr
- https://shadertoyunofficial.wordpress.com/2019/01/02/programming-tricks-in-shadertoy-glsl/