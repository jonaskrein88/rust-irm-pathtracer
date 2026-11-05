## Path tracer with radiance cached global illumination written in Rust

A long running on/off project where I play around with some rendering concepts.
The main focus of this renderer is the biased global illumination based on VRays irradiance map. At the moment the renderer does not store irradiance (incoming) but radiance (outgoing energy),
meaning view dependant effects like reflections and refractions are not factored into the GI yet.

The radiance map is a KD tree, created by recursively sampling and storing the re-emitted light of the geometry.

Architecture built at [Sooii](https://sooii.de/) for Qlocktwo

![beauty](images/QLT_moon.png)

Deug view of the radiance sample distribution.
![radiance_map](images/QLT_moon_RadianceMap.png)

![beauty](images/QLT_Platin.png)

![beauty](images/QLT_RoundRoom.png)
