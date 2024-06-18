#!/bin/bash

# clone concrete lib and patch it
git clone https://github.com/zama-ai/concrete.git
cd concrete
git checkout 3d0727b845154891559eef504e845517d429e5ae
git apply ../concrete.patch
