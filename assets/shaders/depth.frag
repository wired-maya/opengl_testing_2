#version 330 core

void main()
{             
    // Nothing happens because we only calculate depth in this shader
    gl_FragDepth = 0.1;
}