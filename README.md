Macroquad tamogotchi little game
===

You will raise a birds in this game. Probably will add something like motivation tracking (like habitica) or similar kind of stuff.
I try to use lottie animations in there. I wanted to use lottie-rs but there are too much dependencies. I hope I can make my own renderer not complex and able to render stuff that I animate in Synfig/Glaxnimate

# TODO:
- fix metal rendering by providing index_buffer
- nanoserde: use default instead of None when skip fields

# Resources:
- [lottie-rs](https://github.com/zimond/lottie-rs) - Ported module structure to nanoserde
- [contrast_renderer](https://github.com/Lichtso/contrast_renderer) - Stole most of it for path_rendering module
- [How to draw a complex shape using a stencil buffer](http://web.archive.org/web/20240118160026/https://www.glprogramming.com/red/chapter14.html#name13)
- [Svg to shadertoy generator](https://gist.github.com/Ninja-Koala/74fa7652fb4de248949ce1e27b989c14) - currently stealing anti-aliasing from it
- https://handmade.network/forums/t/8799-anti-aliasing_in_fragment_shader
- https://www.shadertoy.com/view/DtXcRr
- https://shadertoyunofficial.wordpress.com/2019/01/02/programming-tricks-in-shadertoy-glsl/