# Headless bench comparison

Compare terrain silhouette, boot framing, material palette, snow caps, spruce crowns, and the
camp's warm pool. The two images intentionally differ in these ways: the bench is a Cycles path
tracer while the client is a Bevy/wgpu rasterizer; the headless client is 1280x720 and the bench
is 960x540 (both 16:9); the bench has no aurora, stars, distance fog, or `rim_level`; and their
exports/captures landed at different ticks, so dwarf positions differ while terrain does not.

The bench contains one exposed-face mesh sourced only from the exported wire snapshot. The client
frame is a `gui --headless --capture` output from the same default seed and boot framing.
