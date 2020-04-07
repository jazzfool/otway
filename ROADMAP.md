# Feature Roadmap

## Accessibility

The primary target platform for this is Windows. MacOS will accessibility will be implemented after Windows. Linux systems are ambiguous on this regard since each desktop environment has their own standards.

First step would be to look into MSAA (Microsoft Active Accessibility) and IAccessible2.

This also includes full support for keyboard shortcuts.

## Animations

This is more open-ended and doesn't require deep integration but it's more of a quality of life enhancement anyway, so it would be nice to have it as a batteries-included solution.

## Internationalization and Localization

Thankfully, the `reclutch::display` API was built with this in mind. Realistically, any font shaping engine can be plugged into the text render command.

Further, there a some localization libraries written in Rust which can be used. It might be a good idea to build a generic interface which the user can implement for the localization library that they're using.

## Mutliple Windows

Shouldn't be too difficult to implement as long as `winit` plays nice.

# Possible Improvements

## Separate Render Thread

If the update thread didn't have to block on the rendering, we might see some good improvements to performance.

Not too difficult to implement, given that render commands can be expressed as a simple list which can then be sent to the other thread for processing.
