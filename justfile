set windows-powershell := true


# compile release
build:
	cargo build --release


# render qlt platin
platin:
	target/release/raytracer.exe models/scene_QLT_Platin.toml

# render qlt moon
moon:
	target/release/raytracer.exe models/scene_QLT_Moon.toml

# render qlt round room
round:
	target/release/raytracer.exe models/scene_QLT_RoundRoom.toml


# show irradiance map samples
moon_ir:
	target/release/raytracer.exe models/scene_QLT_Moon_ir.toml


# show direct light only
moon_direct:
	target/release/raytracer.exe models/scene_QLT_Moon_direct.toml

