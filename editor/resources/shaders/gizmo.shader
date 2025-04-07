(
    name: "GizmoShader",

    resources: [
        (
            name: "properties",
            kind: PropertyGroup([
                (
                    name: "diffuseColor",
                    kind: Color(r: 255, g: 255, b: 255, a: 255),
                ),
            ]),
            binding: 0
        ),
        (
            name: "fyrox_instanceData",
            kind: PropertyGroup([
                // Autogenerated
            ]),
            binding: 1
        ),
    ],

    disabled_passes: ["GBuffer", "DirectionalShadow", "PointShadow", "SpotShadow"],

    passes: [
        (
            name: "Forward",
            draw_parameters: DrawParameters(
                cull_face: None,
                color_write: ColorMask(
                    red: true,
                    green: true,
                    blue: true,
                    alpha: true,
                ),
                depth_write: true,
                stencil_test: None,
                depth_test: Some(Less),
                blend: Some(BlendParameters(
                    func: BlendFunc(
                        sfactor: SrcAlpha,
                        dfactor: OneMinusSrcAlpha,
                        alpha_sfactor: SrcAlpha,
                        alpha_dfactor: OneMinusSrcAlpha,
                    ),
                    equation: BlendEquation(
                        rgb: Add,
                        alpha: Add
                    )
                )),
                stencil_op: StencilOp(
                    fail: Keep,
                    zfail: Keep,
                    zpass: Keep,
                    write_mask: 0xFFFF_FFFF,
                ),
                scissor_box: None
            ),
            vertex_shader:
               r#"
                layout(location = 0) in vec3 vertexPosition;

                void main()
                {
                    gl_Position = fyrox_instanceData.worldViewProjection * vec4(vertexPosition, 1.0);
                }
               "#,

           fragment_shader:
               r#"
                out vec4 FragColor;

                void main()
                {
                    FragColor = properties.diffuseColor;

                    // Pull depth towards near clipping plane so the gizmo will be drawn on top
                    // of everything, but its parts will be correctly handled by depth test.
                    gl_FragDepth = gl_FragCoord.z * 0.001;
                }
               "#,
        ),
    ],
)