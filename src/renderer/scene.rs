
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
        materials: Vec<TOML_Material>,
        objects:   Vec<TOML_Geometry>,
        environment_map: Option<String>,
        environment_rotation: Option<f32>,
        environment_strength: Option<f32>,
        render_mode : Option<String>,
        gi_enabled : Option<bool>,
        dof_enabled : Option<bool>,
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
    struct TOML_Geometry
    {
        file: String,
        materials : Vec<String>
    }

    #[derive(Debug, Deserialize)]
    struct TOML_Material
    {
        name: String,
        color: [f32;3],
        roughness : f32,
        reflection_enabled : bool,
        ior : f32
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
        if let Some(x) = config.environment_rotation{
            renderer.settings.environment_map_rotation = x;
        }
        if let Some(x) = config.gi_enabled{
            renderer.settings.gi_enabled = x;
        }
        if let Some(x) = config.dof_enabled{
            renderer.settings.dof_enabled = x;
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



        // collect the material names here
        // to get the material indexes later
        let mut material_registry : Vec<String> = Vec::new();

        // create materials
        for mtl in &config.materials
        { 
            let color = Color::new(mtl.color[0], mtl.color[1], mtl.color[2]);
            let mut new_material = Material::new()
                .set_name(&mtl.name)
                .set_reflection_enabled(mtl.reflection_enabled)
                .set_roughness(mtl.roughness)
                .set_ior(mtl.ior)
                .set_color(color);

            let _id = renderer.add_material(new_material);

            // store material name
            material_registry.push(mtl.name.clone());
        }

        // create objects
        for obj in &config.objects
        { 
            let path = filepath.join(&obj.file);
            let mesh = OBJ_Importer::open(path.to_str().unwrap()).unwrap();
		
            let mut geo = Geo::from_mesh(mesh);
            let assignments = obj.materials.len();
            let slots = geo.materials.len();
            if assignments < slots {panic!("Not enough materials assigned. Mesh has {slots:?} slots");}
            if assignments > slots {println!("More materials assigned than available slots - ignoring");}

            // assign the materials that have are owned by the renderer now
            for (i,mat_name) in obj.materials.iter().enumerate()
            {
                if let Some(identifier) = material_registry.iter().position(|x| x==mat_name)
                {
                    geo = geo.assign_material(i, identifier);
                }
                else {println!("Could not assign material {mat_name}");}
            }
            renderer.add(geo);
        }
        


        
    }
}
