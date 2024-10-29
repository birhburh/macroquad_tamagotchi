use miniquad::*;
use window::screen_size;

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
}

struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
    ctx: Box<dyn RenderingBackend>,
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        #[rustfmt::skip]
        let vertices: [Vertex; 6] = [
            Vertex { pos : [ -1.0, -1.0 ] },
            Vertex { pos : [ -1.0,  1.0 ] },
            Vertex { pos : [  1.0,  1.0 ] },
            Vertex { pos : [  1.0,  1.0 ] },
            Vertex { pos : [  1.0, -1.0 ] },
            Vertex { pos : [ -1.0, -1.0 ] },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 1, 2, 3, 4, 5];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };

        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader::meta(),
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        );

        Stage {
            pipeline,
            bindings,
            ctx,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.ctx.begin_default_pass(Default::default());

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx
            .apply_uniforms(UniformsSource::table(&shader::Uniforms {
                i_resolution: screen_size(),
            }));
        self.ctx.draw(0, 6, 1);
        self.ctx.end_render_pass();

        self.ctx.commit_frame();
    }
}

fn main() {
    let mut conf = conf::Conf::default();
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    conf.platform.apple_gfx_api = if metal {
        conf::AppleGfxApi::Metal
    } else {
        conf::AppleGfxApi::OpenGl
    };

    miniquad::start(conf, move || Box::new(Stage::new()));
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    precision lowp float;
    uniform vec2 iResolution;

    #define ZERO int(min(0.0,iResolution.x))

    float border;

    // Modified from http://tog.acm.org/resources/GraphicsGems/gems/Roots3And4.c
    // Credits to Doublefresh for hinting there
    int solve_quadric(vec2 coeffs, inout vec2 roots){

        // normal form: x^2 + px + q = 0
        float p = coeffs[1] / 2.;
        float q = coeffs[0];

        float D = p * p - q;

        if (D < 0.){
            return 0;
        }
        else{
            roots[0] = -sqrt(D) - p;
            roots[1] = sqrt(D) - p;

            return 2;
        }
    }

    //From Trisomie21
    //But instead of his cancellation fix i'm using a newton iteration
    int solve_cubic(vec3 coeffs, inout vec3 r){

        float a = coeffs[2];
        float b = coeffs[1];
        float c = coeffs[0];

        float p = b - a*a / 3.0;
        float q = a * (2.0*a*a - 9.0*b) / 27.0 + c;
        float p3 = p*p*p;
        float d = q*q + 4.0*p3 / 27.0;
        float offset = -a / 3.0;
        if(d >= 0.0) { // Single solution
            float z = sqrt(d);
            float u = (-q + z) / 2.0;
            float v = (-q - z) / 2.0;
            u = sign(u)*pow(abs(u),1.0/3.0);
            v = sign(v)*pow(abs(v),1.0/3.0);
            r[0] = offset + u + v;

            //Single newton iteration to account for cancellation
            float f = ((r[0] + a) * r[0] + b) * r[0] + c;
            float f1 = (3. * r[0] + 2. * a) * r[0] + b;

            r[0] -= f / f1;

            return 1;
        }
        float u = sqrt(-p / 3.0);
        float v = acos(-sqrt( -27.0 / p3) * q / 2.0) / 3.0;
        float m = cos(v), n = sin(v)*1.732050808;

        //Single newton iteration to account for cancellation
        //(once for every root)
        r[0] = offset + u * (m + m);
        r[1] = offset - u * (n + m);
        r[2] = offset + u * (n - m);

        vec3 f = ((r + a) * r + b) * r + c;
        vec3 f1 = (3. * r + 2. * a) * r + b;

        r -= f / f1;

        return 3;
    }

    float quadratic_bezier_normal_iteration(float t, vec2 a0, vec2 a1, vec2 a2){
        //horner's method
        vec2 a_1=a1+t*a2;

        vec2 uv_to_p=a0+t*a_1;
        vec2 tang=a_1+t*a2;

        float l_tang=dot(tang,tang);
        return t-dot(tang,uv_to_p)/l_tang;
    }

    float quadratic_bezier_dis_approx_sq(vec2 uv, vec2 p0, vec2 p1, vec2 p2){
        vec2 a2 = p0 - 2. * p1 + p2;
        vec2 a1 = -2. * p0 + 2. * p1;
        vec2 a0 = p0 - uv;

        float d0 = 1e38;

        float t;
        vec3 params=vec3(0,.5,1);

        if(all(lessThan(uv,max(max(p0,p1),p2)+border)) && all(greaterThan(uv,min(min(p0,p1),p2)-border))){
            for(int i=ZERO;i<3;i++){
                t=params[i];
                for(int j=ZERO;j<3;j++){
                    t=quadratic_bezier_normal_iteration(t,a0,a1,a2);
                }
                t=clamp(t,0.,1.);
                vec2 uv_to_p=(a2*t+a1)*t+a0;
                d0=min(d0,dot(uv_to_p,uv_to_p));
            }
        }

        return d0;
    }

    float cubic_bezier_normal_iteration(float t, vec2 a0, vec2 a1, vec2 a2, vec2 a3){
        //horner's method
        vec2 a_2=a2+t*a3;
        vec2 a_1=a1+t*a_2;
        vec2 b_2=a_2+t*a3;

        vec2 uv_to_p=a0+t*a_1;
        vec2 tang=a_1+t*b_2;

        float l_tang=dot(tang,tang);
        return t-dot(tang,uv_to_p)/l_tang;
    }

    float cubic_bezier_dis_approx_sq(vec2 uv, vec2 p0, vec2 p1, vec2 p2, vec2 p3){
        vec2 a3 = (-p0 + 3. * p1 - 3. * p2 + p3);
        vec2 a2 = (3. * p0 - 6. * p1 + 3. * p2);
        vec2 a1 = (-3. * p0 + 3. * p1);
        vec2 a0 = p0 - uv;

        float d0 = 1e38;

        float t;
        vec3 params=vec3(0,.5,1);

        if(all(lessThan(uv,max(max(p0,p1),max(p2,p3))+border)) && all(greaterThan(uv,min(min(p0,p1),min(p2,p3))-border))){
            for(int i=ZERO;i<3;i++){
                t=params[i];
                for(int j=ZERO;j<3;j++){
                    t=cubic_bezier_normal_iteration(t,a0,a1,a2,a3);
                }
                t=clamp(t,0.,1.);
                vec2 uv_to_p=((a3*t+a2)*t+a1)*t+a0;
                d0=min(d0,dot(uv_to_p,uv_to_p));
            }
        }

        return d0;
    }

    //segment_dis_sq by iq
    float length2( vec2 v ) { return dot(v,v); }

    float segment_dis_sq( vec2 p, vec2 a, vec2 b ){
        vec2 pa = p-a, ba = b-a;
        float h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
        return length2( pa - ba*h );
    }

    int segment_int_test(vec2 uv, vec2 p0, vec2 p1){
        p0-=uv;
        p1-=uv;

        int ret;

        if(p0.y*p1.y<0.){
            vec2 nor=p0-p1;
            nor=vec2(nor.y,-nor.x);

            float sgn;

            if(p0.y>p1.y){
                sgn=1.;
            }
            else{
                sgn=-1.;
            }

            if(dot(nor,p0)*sgn<0.){
                ret=0;
            }
            else{
                ret=1;
            }
        }
        else{
            ret=0;
        }

        return ret;
    }

    int quadratic_bezier_int_test(vec2 uv, vec2 p0, vec2 p1, vec2 p2){

        float qu = (p0.y - 2. * p1.y + p2.y);
        float li = (-2. * p0.y + 2. * p1.y);
        float co = p0.y - uv.y;

        vec2 roots = vec2(1e38);
        int n_roots = solve_quadric(vec2(co/qu,li/qu),roots);

        int n_ints = 0;

        for(int i=ZERO;i<n_roots;i++){
            if(roots[i] >= 0. && roots[i] <= 1.){
                float x_pos = p0.x - 2. * p1.x + p2.x;
                x_pos = x_pos * roots[i] + -2. * p0.x + 2. * p1.x;
                x_pos = x_pos * roots[i] + p0.x;

                if(x_pos > uv.x){
                    n_ints++;
                }
            }
        }

        return n_ints;
    }

    int cubic_bezier_int_test(vec2 uv, vec2 p0, vec2 p1, vec2 p2, vec2 p3){

        float cu = (-p0.y + 3. * p1.y - 3. * p2.y + p3.y);
        float qu = (3. * p0.y - 6. * p1.y + 3. * p2.y);
        float li = (-3. * p0.y + 3. * p1.y);
        float co = p0.y - uv.y;

        vec3 roots = vec3(1e38);
        int n_roots;

        int n_ints=0;

        if(uv.x<min(min(p0.x,p1.x),min(p2.x,p3.x))){
            if(uv.y>=min(p0.y,p3.y) && uv.y<=max(p0.y,p3.y)){
                n_ints=1;
            }
        }
            else{
            if(abs(cu) < .0001){
                n_roots = solve_quadric(vec2(co/qu,li/qu),roots.xy);
            }
            else{
                n_roots = solve_cubic(vec3(co/cu,li/cu,qu/cu),roots);
            }

            for(int i=ZERO;i<n_roots;i++){
                if(roots[i] >= 0. && roots[i] <= 1.){
                    float x_pos = -p0.x + 3. * p1.x - 3. * p2.x + p3.x;
                    x_pos = x_pos * roots[i] + 3. * p0.x - 6. * p1.x + 3. * p2.x;
                    x_pos = x_pos * roots[i] + -3. * p0.x + 3. * p1.x;
                    x_pos = x_pos * roots[i] + p0.x;

                    if(x_pos > uv.x){
                        n_ints++;
                    }
                }
            }
        }

        return n_ints;
    }

    int modI(int a,int b) {
        float m=float(a)-floor((float(a)+0.5)/float(b))*float(b);
        return int(floor(m+0.5));
    }

    float path0_dis_sq(vec2 uv){
        float dis_sq=1e38;

        int num_its=0;

        vec2 p[4];
        p[0] = vec2(-0.4,-0.144);
        p[1] = vec2(-0.144,0.144);
        p[2] = vec2(0.208,0.112);
        p[3] = vec2(0.4,-0.144);

        ivec4 c_bez[1];
        c_bez[0] = ivec4(0,1,2,3);

        if(all(lessThan(uv,vec2(0.4,0.144)+border)) && all(greaterThan(uv,vec2(-0.4,-0.144)-border))){
            for(int i=ZERO;i<1;i++){
                dis_sq=min(dis_sq,cubic_bezier_dis_approx_sq(uv,p[c_bez[i][0]],p[c_bez[i][1]],p[c_bez[i][2]],p[c_bez[i][3]]));
                num_its+=cubic_bezier_int_test(uv,p[c_bez[i][0]],p[c_bez[i][1]],p[c_bez[i][2]],p[c_bez[i][3]]);
            }
        }

        float sgn=1.;

        if(modI(num_its,2)==1){
            sgn=-1.;
        }

        return sgn*dis_sq;
    }

    void main() {
        border=1./iResolution.x;

        vec2 uv=gl_FragCoord.xy/iResolution.xy;
        uv-=.5;
        uv.y*=iResolution.y/iResolution.x;

        float dis_sq=1e38;

        if(all(lessThan(uv,vec2(0.4,0.144)+border)) && all(greaterThan(uv,vec2(-0.4,-0.144)-border))){
            dis_sq=min(dis_sq,path0_dis_sq(uv));
        }

        float dis=sign(dis_sq)*sqrt(abs(dis_sq));

        gl_FragColor=vec4(smoothstep(-border, border, dis));
    }
    "#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
    };

    struct Uniforms
    {
        float2 iResolution;
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy, 0.0, 1.0);

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]],
                                   constant Uniforms& uniforms [[buffer(0)]])
    {
        float2 uv = in.position.xy/uniforms.iResolution.xy * 2. - 1.;
        uv.x *= uniforms.iResolution.x / uniforms.iResolution.y;

        float2 position = float2(.0, 0.);
        float radius = .7;
        float width = .1;
        float dist = distance(uv, position);
        float4 color = float4(.8, .7, 1., 1.);

        float aa = 2.0 / min(uniforms.iResolution.x, uniforms.iResolution.y);

        // Our alpha channel
        float alpha = 1. - smoothstep(width - aa, width + aa, abs(dist - radius));

        color.a *= alpha;
        color.rgb *= alpha;

        return color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("iResolution", UniformType::Float2),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub i_resolution: (f32, f32),
    }
}
