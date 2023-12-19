function output = hueThreshold(path, bounds)
image = imread(path);
hsvImage = rgb2hsv(image);
hue = hsvImage(:, :, 1);
output = multithresh(hue, bounds) * 360;
end