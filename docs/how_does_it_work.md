# How does Oblichey work?

This document aims to give an overview of how Oblichey works.

## How does one do facial recognition

### Choosing a camera

There are several things that need to be thought of when doing anything related
to facial authentication. The first one is that using a traditional RGB camera
is a risky approach. While they will do just fine for recognition, they are
very easily fooled with a picture or a video which makes them a bad choice for
authentication. Given this problem, infrared (IR) cameras are the preferred
choice for authentication. They can still be fooled with a printed picture, but
they cannot be fooled with a face on a screen.

The question then is how do we avoid being fooled by a printed picture. The
answer is _liveness detection_ (this has not been implemented yet, see
https://github.com/SimonBrandner/oblichey/issues/6) - a technique used to
detected if a given face is "alive". This is done, for example, by detecting
eye movement.

In summary, an IR camera prevents the program from being fooled by a video,
while liveness detection prevents it from being fooled with a printed picture.

### Neural networks

To perform the individual tasks such as face detection and recognition, neural
networks may be used. Oblichey uses models from
[FaceONNX](https://github.com/FaceONNX/FaceONNX) and imports them using the
[Burn](https://burn.dev/) deep learning framework.

The problem of using neural networks for face recognition is quite an
interesting one since in comparison to some other tasks (like digit
recognition) there are no clear categories (that is unless we have a predefined
list of people for the network to be able to recognize which would be hugely
impractical). So instead of the network's output being something like "this is
John" or "this is not John", it outputs an n-dimensional vector (called
_embedding_) which represents a given face. If the angle between two of these
embedding vectors is small, the faces are similar/the same, if the angle is
large, each face is different. (Interestingly, networks which are trained to
divide object into distinct categories develop a layer before the last which
works in a similar manner to the last layer of networks which have embeddings
as an output.) To train a network to do this, triplets of images are used. Two
of these images include the same face and one includes a face different from
the other two, the loss function then "encourages" the network to give the two
images of the same face a similar embedding and the image with the different
face a different embedding.

It is good to note that most face detection and recognition networks are
trained on RGB images while we are using an IR camera whose output is in the
form of gray-scale images. It turns out that if you convert the gray-scale
image to an RGB image (by taking the gray-scale value and making that the red,
green and blue value), the network will work just fine. That being said there
certainly are benefits to be had (performance at the very least since now we
are essentially processing everything 3 times, once for each of red, green and
blue) from using a network trained on a gray-scale dataset.

## Code structure

Oblichey is split into two parts `oblichey-cli` and `oblichey-pam-module`, the
latter compiles to a shared C library which PAM then uses. Its code is rather
simple as it just runs the `oblichey-cli` binary determining the authentication
result based on its return value. It would be better to split most of
`oblichey-cli` into some sort of shared library which could then be used both
by the CLI and the PAM module but as far as I know this simply is not possible
in Rust at the current time.

For most commands, the CLI will create several threads, one for getting frames
from the camera, second for processing those frames and a third optional one
for the GUI. The need for multiple threads comes from the fact that the neural
networks are not fast enough to process many frames per second, so we let the
processing code live in its own thread and always get the latest frames when it
has finished processing while the GUI thread can display most, if not all, of
the frames coming from the camera.

Currently (this might change in the future), we have two "processors" in the
processing thread. This first one (`FrameProcessor`) takes a frame as an input
and returns detected faces along with their embeddings. This return value is
then passed to the second one (`FaceProcessor` which is a trait) which,
depending on the implementation, may either check whether any of the faces
supplied are known or scan a new face.
