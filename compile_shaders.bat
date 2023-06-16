cd res/shaders/src
for %%a in (*) do C:/VulkanSDK/1.3.243.0/Bin/glslc.exe %%a -o ../bin/%%a.spv