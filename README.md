# stegostl
Embedding data (steganography) in 3D STL models

# Introduction
Can you spot a difference between these two Benchy models?

<img src="docs/img_benchy_compare.png" alt="Two benchy models side by side" width="90%"/>

The model on the right has a puffin in it, but you wont spot it in a model viewer or notice that the file is any different than the default Benchy via simple review of the file.

<img src="demo_files/puffin.jpg" alt="Puffin" width="30%"/>
<img src="docs/img_benchy_files_compare.png" alt="Filename comparison of 3DBenchy files with the same size" /> 

Using the tools in this project, [this image](demo_files/puffin.jpg) was embedded within the geometry data of a standard-issue [3D Benchy](demo_files/3DBenchy.stl), creating a [new STL model file](demo_files/3DBenchy_Embedded_Puffin.stl) that for all intents and purposes is still a perfectly functional Benchy model. The new STL is the exact same size as the original file, and will load in any model viewer, slicer or 3D Modelling program with no indication that there is a JPEG image buried in its data -- unless you know how to look for it.

This is an implementation of the concept of [steganography](https://en.wikipedia.org/wiki/Steganography) within the context of a STL 3D model file - hiding arbitrary information within the structure of the STL format, so it can be retrieved by a process that is capable of reconstructing it.

## Prove it
You can download or build the Rust project here and use the `data` executable to extract the image from `3DBenchy_Embedded_Puffin.stl` - you just need to know a few pieces of info about the enhanced STL file - the "bit depth" the data is encoded at (more on this later in this doc), which in this case is `5`, and the filename you want to output to (since you know you're expecting a JPEG image, might as well name it a `.jpg` file.)

    # data decode demo_files/3DBenchy_Embedded_Puffin.stl puffin_out.jpg 5
        File: demo_files/3DBenchy_Embedded_Puffin.stl
        Tris: 225706
        Vertices: 112662
        Header read, payload bytes: 204881
        Writing 204881 bytes of data to output file puffin_out.jpg
        Decode complete.
        
    # md5sum puffin_out.jpg demo_files/puffin.jpg
        bfa4248c0b641e814a51a95530865334  puffin_out.jpg
        bfa4248c0b641e814a51a95530865334  demo_files/puffin.jpg



# Inspiration


#### Attributions and License
Project code is licensed under the MIT License.

Please see [this file](demo_files/attributions_and_licenses.txt) for all license and attribution notes for non-original files used in this project's demo files and this documentation
