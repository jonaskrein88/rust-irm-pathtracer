
pub mod SceneImporter
{
    use crate::renderer::{Renderer,Material,Color,Geo,OBJ_Importer,Texture,RenderMode};
    use serde::Deserialize;
    use std::fs;
    use regex::Regex;
    use std::path::{Path,PathBuf};

    #[derive(Debug, Deserialize)]
    struct Config
    {
        camera:    TOML_Camera,
        textures:  Vec<TOML_Texture>,
        materials: Vec<TOML_Material>,
        objects:   Vec<TOML_Geometry>,
        environment_map: Option<String>,
        environment_rotation_degrees: Option<f32>,
        environment_strength: Option<f32>,
        render_mode : Option<String>,
        gi_enabled : Option<bool>,
        dof_enabled : Option<bool>,
        output_file : Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct TOML_Camera
    {
        blender_cam: String,
        focal_length: Option<f32>,
        image_size : Option<[u32;2]>,
        render_height : Option<u32>,
        aperture_radius: Option<f32>,
        focus_dist: Option<f32>,
    }


    #[derive(Debug, Deserialize)]
    struct TOML_Texture
    {
        name: String,
        file: String
    }

    #[derive(Debug, Deserialize)]
    struct TOML_Material
    {
        name: String,
        color: Option<[f32;3]>,
        texture_diffuse: Option<String>,
        texture_normals:  Option<String>,
        texture_roughness:  Option<String>,
        texture_roughness_strength:  Option<f32>,
        invert_roughness_map: Option<bool>,
        normal_strength:  Option<f32>,
        roughness : Option<f32>,
        reflection_enabled : Option<bool>,
        ior : Option<f32>
    }

    #[derive(Debug, Deserialize)]
    struct TOML_Geometry
    {
        file: String,
        materials : Vec<String>
    }


    // unpack the printed matrix string to a 2d array
    fn parse_blender_matrix(matrix_str: &str) -> Result<[[f32; 4]; 4], String>
    {
        let re = Regex::new(r"[-+]?\d*\.\d+|\d+").unwrap();
        
        // find all number matches in the string
        let mut numbers = re.find_iter(matrix_str)
            .map(|m| m.as_str())
            .skip(2); // skip the 4x4 part

        let mut matrix = [[0.0; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                if let Some(num_str) = numbers.next() {
                    matrix[r][c] = num_str.parse::<f32>()
                        .map_err(|_| format!("Failed to parse float: {}", num_str))?;
                } else {
                    return Err("Not enough numbers found in the matrix string".to_string());
                }
            }
        }

        Ok(matrix)
    }



    fn find_texture_by_name(renderer:&Renderer, name:&str) -> usize{
        if let Some(identifier) = renderer.textures.iter().position(|x| x.name==name){
            identifier
        }
        else {
            panic!("could not find a texture with name {name}");
        }
    }

    fn find_material_by_name(renderer:&Renderer, name:&str) -> usize{
        if let Some(identifier) = renderer.materials.iter().position(|x| x.name==name){
            identifier
        }
        else {
            panic!("could not find material with name {name}");
        }
    }
    


    pub fn import_scene(renderer : &mut Renderer, mut filepath:PathBuf)
    {
        let contents = fs::read_to_string(filepath.to_str().unwrap())
            .expect("Something went wrong while trying to access the scene file");

        // deserialize the String into Config struct
        let config: Config = toml::from_str(&contents)
            .expect("Failed to parse TOML");
        
        // println!("{config:?}");
        

        filepath.pop();

        // environment
        if let Some(file) = config.environment_map {
		    let hdri_path = filepath.join(file);
            let hdri = Texture::open_exr(hdri_path.to_str().unwrap());
            renderer.set_environment(hdri);
        }

        if let Some(x) = config.environment_strength{
            renderer.settings.environment_map_multiplier = x;
        }
        

        // recreating the rotation of a blender environment
        let degrees = match config.environment_rotation_degrees
        {
            Some(x) => x,
            None => 0.0,
        };
        //println!("environment rotation set to {0} degrees", degrees);
        let fraction = (-degrees - 90.0) / 360.0;
        renderer.settings.environment_map_rotation = fraction.rem_euclid(1.0);
        



        if let Some(x) = config.gi_enabled{
            renderer.settings.gi_enabled = x;
        }
        if let Some(x) = config.dof_enabled{
            renderer.settings.dof_enabled = x;
        }
        if let Some(x) = config.output_file{
            renderer.settings.output_file = Some(x);
        }


        if let Some(x) = config.render_mode
        {
            match x.as_str()
            {
                "beauty"         => renderer.settings.render_mode = RenderMode::Beauty,
                "gi"             => renderer.settings.render_mode = RenderMode::Debug_GI,
                "gi_raw"         => renderer.settings.render_mode =RenderMode::Debug_Raw_GI,
                "irradiance_map" => renderer.settings.render_mode = RenderMode::Debug_IrradianceMap,
                "ir_samples"     => renderer.settings.render_mode = RenderMode::Debug_IrradianceMap_Samples,
                "normals"        => renderer.settings.render_mode = RenderMode::Debug_Normals,
                "uvs"            => renderer.settings.render_mode = RenderMode::Debug_UVs,
                "reflections"    => renderer.settings.render_mode = RenderMode::Debug_Reflections,
                "focus"          => renderer.settings.render_mode = RenderMode::Debug_Focus,
                _                => panic!("render mode not supported")

            }
        }

        
        // try and set the camera to blenders    
        if let Ok(values) = parse_blender_matrix(&config.camera.blender_cam){
            renderer.camera.from_blender_transform(&values);            
        }
        else {panic!("Could not set camera from blender xform");}

        if let Some(size) = config.camera.image_size{
            renderer.set_image_size(size[0],size[1]);
        }
        
        if let Some(x) = config.camera.focal_length{
            renderer.camera.focal_length = x;
        }
        if let Some(x) = config.camera.focus_dist{
            renderer.camera.focus_dist = x;
        }
        if let Some(x) = config.camera.aperture_radius{
            renderer.camera.aperture_radius = x;
        }
        

        // Textures

        for tex in &config.textures
        { 
		    let tex_path = filepath.join(&tex.file);
            let mut new_texture = Texture::open(tex_path.to_str().unwrap());
            new_texture.name = tex.name.clone();

            let _id = renderer.add_texture(new_texture);

            // store texture name
            println!("texture {0} stored  {_id:?}", tex.name);
        }



        // Materials

        // collect the material names here
        // to get the material indexes later
        //let mut material_registry : Vec<String> = Vec::new();

        // create materials
        for mtl in &config.materials
        { 

            let mut new_material = Material::new()
                .set_name(&mtl.name);
           
            if let Some(color) = mtl.color {
                new_material = new_material.set_color(Color::new(color[0], color[1], color[2]));
            }
            if let Some(nstrength) = mtl.normal_strength{
                new_material.normal_strength = nstrength;
            }
            if let Some(ior) = mtl.ior{
                new_material.ior = ior;
            }
            if let Some(roughness) = mtl.roughness{
                new_material.roughness = roughness;
            }
            if let Some(reflections) = mtl.reflection_enabled{
                new_material = new_material.set_reflection_enabled(reflections);
            
            }
            if let Some(r) = mtl.texture_roughness_strength{
                println!("setting roughness tex amount to {r}");
                new_material.texture_roughness_strength = r;
            }
            if let Some(inv) = mtl.invert_roughness_map{
                new_material.invert_roughness_map = inv;
            }
            if let Some(ref tex_name) = mtl.texture_diffuse{
                let id = find_texture_by_name(&renderer, tex_name);
                new_material = new_material.set_texture_diffuse(id);
                //println!("assigned diffuse texture {tex_name} to {0}", mtl.name);
            }
            if let Some(ref tex_name) = mtl.texture_normals{
                let id = find_texture_by_name(&renderer, tex_name);
                new_material = new_material.set_texture_normals(id);
            }
            if let Some(ref tex_name) = mtl.texture_roughness{
                let id = find_texture_by_name(&renderer, tex_name);
                new_material = new_material.set_texture_roughness(id);
            }
            let _id = renderer.add_material(new_material);
            println!("material {0} stored  {_id:?}", mtl.name);

            // store material name
            //material_registry.push(mtl.name.clone());
        }

        // create objects
        for obj in &config.objects
        { 
            let mesh_path = filepath.join(&obj.file);
            let mesh = OBJ_Importer::open(mesh_path.to_str().unwrap()).unwrap();
		
            let mut geo = Geo::from_mesh(mesh);
            let assignments = obj.materials.len();
            let slots = geo.materials.len();
            if assignments < slots {println!("Not enough materials assigned. {0} materials are set to default", slots-assignments);}
            if assignments > slots {println!("More materials assigned than available slots - ignoring");}



            // assign the materials that are owned by the renderer now
            // find it by name and assign material identifier
            for (slot,mat_name) in obj.materials.iter().enumerate()
            {
                let index = find_material_by_name(&renderer,mat_name);
                geo = geo.assign_material(slot, index);
                //println!("trying to find {mat_name}, found {0} at index {index}", renderer.materials[index].name);
                //println!("assigning to material id {slot}");
            }
            renderer.add(geo);
        }
        


        
    }
}
