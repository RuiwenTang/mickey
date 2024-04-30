pub(crate) struct Command<'a> {
    pipeline: &'a wgpu::RenderPipeline,
    vertex_buffer: wgpu::BufferSlice<'a>,
    index_buffer: wgpu::BufferSlice<'a>,
    draw_count: u32,

    groups: Vec<wgpu::BindGroup>,
}

impl<'a> Command<'a> {
    pub(crate) fn new(
        pipeline: &'a wgpu::RenderPipeline,
        vertex_buffer: wgpu::BufferSlice<'a>,
        index_buffer: wgpu::BufferSlice<'a>,
        draw_count: u32,
        groups: Vec<wgpu::BindGroup>,
    ) -> Self {
        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            draw_count,
            groups,
        }
    }

    pub(crate) fn run(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);

        pass.set_vertex_buffer(0, self.vertex_buffer.clone());
        pass.set_index_buffer(self.index_buffer.clone(), wgpu::IndexFormat::Uint32);

        for (i, group) in self.groups.iter().enumerate() {
            pass.set_bind_group(i as u32, group, &[]);
        }

        pass.draw_indexed(0..self.draw_count, 0, 0..1);
    }
}

pub(crate) struct CommandList<'a> {
    commands: Vec<Command<'a>>,
}

impl<'a> CommandList<'a> {
    pub(crate) fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub(crate) fn add_command_list(&mut self, commands: Vec<Command<'a>>) {
        self.commands.extend(commands);
    }

    pub(crate) fn run(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        for command in &self.commands {
            command.run(pass);
        }
    }
}

impl<'a> FromIterator<Command<'a>> for CommandList<'a> {
    fn from_iter<T: IntoIterator<Item = Command<'a>>>(iter: T) -> Self {
        let mut list = Self::new();
        list.add_command_list(iter.into_iter().collect());
        list
    }
}
