# Tiffiny

## TODO:

Legend:
[ ] = not started
[~] = in progress
[x] = completed

## 0. Vision / Planning

[ ] Define exact project identity
[ ] Define UI direction
[ ] Define supported operating systems
[ ] Define architecture philosophy
[ ] Define conversion philosophy
[ ] Define project scope boundaries
[ ] Define databending workflow
[ ] Define reinterpretation workflow

### Branding

[ ] Finalize name
[ ] Design logo
[ ] Design icon
[ ] Design splash screen
[ ] Design UI theme system
[ ] Design typography system
[ ] Define color palette

## 1. Repository Structure

### Root Setup

[ ] Update README
[ ] Create roadmap
[ ] Create changelog

### Cargo

[ ] Setup workspace
[ ] Split crates cleanly
[ ] Configure release profiles
[ ] Configure linting
[ ] Configure formatting
[ ] Configure clippy rules

## 2. Core Architecture

[ ] Create modular conversion engine
[ ] Create pipeline architecture
[ ] Create processing scheduler
[ ] Create async task manager
[ ] Create event system
[ ] Create logging system
[ ] Create crash handling
[ ] Create recovery system

### Internal Data Types

[ ] Audio abstraction
[ ] Image abstraction
[ ] Video abstraction
[ ] RAW abstraction
[ ] Binary buffer abstraction
[ ] Metadata abstraction
[ ] Timeline abstraction

### Memory Handling

[ ] Streaming processing
[ ] Large file chunking
[ ] Zero-copy operations
[ ] GPU buffer handling
[ ] Memory pool system

## 3. GUI Foundation

### Windowing

[ ] Main window
[ ] Multi-window support
[ ] Dockable panels
[ ] Resizable panes
[ ] Fullscreen support
[ ] Multi-monitor support

### Layout

[ ] Sidebar system
[ ] Toolbar system
[ ] Inspector panel
[ ] Timeline panel
[ ] Console panel
[ ] Preview panel
[ ] Project explorer
[ ] Context menus

### UX

[ ] Drag and drop
[ ] Keyboard shortcuts
[ ] Command palette
[ ] Undo/redo
[ ] Tooltips
[ ] Notifications
[ ] Progress bars
[ ] Toast system
[ ] Modal system

## 4. File System Features

### Importing

[ ] Audio importing
[ ] Video importing
[ ] Image importing
[ ] RAW importing
[ ] Folder importing
[ ] Recursive imports
[ ] Clipboard imports

### Exporting

[ ] Export dialog
[ ] Export presets
[ ] Export queue
[ ] Batch exporting
[ ] Multi-format export
[ ] Custom filenames
[ ] Metadata export
[ ] ZIP packaging

### File Management

[ ] Recent files
[ ] Project autosave
[ ] Temporary cache system
[ ] Asset relinking
[ ] Duplicate detection

## 5. Audio Engine

### Audio Decoding

[ ] WAV support
[ ] MP3 support
[ ] FLAC support
[ ] OGG support
[ ] AAC support
[ ] AIFF support
[ ] Streaming decode

### Audio Playback

[ ] Play/pause
[ ] Seeking
[ ] Looping
[ ] Scrubbing
[ ] Playback speed
[ ] Volume control
[ ] Audio device selection

### Audio Processing

[ ] FFT analysis
[ ] Spectral transforms
[ ] Resampling
[ ] Normalization
[ ] Phase analysis
[ ] RMS analysis
[ ] Peak detection

## 6. Image Engine

### Image Decoding

[ ] PNG support
[ ] JPG support
[ ] WEBP support
[ ] BMP support
[ ] TIFF support
[ ] GIF support
[ ] HDR support

### Image Processing

[ ] Resize
[ ] Crop
[ ] Rotate
[ ] Flip
[ ] Channel manipulation
[ ] Histogram analysis
[ ] Pixel sorting
[ ] Dithering
[ ] Quantization
[ ] Edge detection

### Image Rendering

[ ] GPU rendering
[ ] Zooming
[ ] Panning
[ ] Tile rendering
[ ] Mipmap generation

## 7. Video Engine

### Video Decoding

[ ] MP4 support
[ ] MKV support
[ ] MOV support
[ ] AVI support
[ ] WEBM support

### Video Features

[ ] Frame extraction
[ ] Timeline scrubbing
[ ] Thumbnail generation
[ ] Frame interpolation
[ ] Motion analysis
[ ] Scene detection

### Rendering

[ ] GPU accelerated playback
[ ] Frame caching
[ ] Hardware decode support

## 8. RAW / Binary Engine

### RAW Tools

[ ] RAW parser
[ ] Byte viewer
[ ] Hex editor
[ ] Entropy visualizer
[ ] Binary heatmap
[ ] Bitplane viewer
[ ] Offset viewer
[ ] Binary inspector
[ ] Structure viewer

### Data Bending

[ ] Controlled corruption
[ ] Random corruption
[ ] Byte shifting
[ ] XOR transforms
[ ] Chunk scrambling
[ ] Header manipulation
[ ] Entropy injection
[ ] Probabilistic corruption
[ ] Byte swapping
[ ] Structure mutation

### RAW Interpretation

[ ] RAW → image
[ ] RAW → audio
[ ] RAW → spectrogram
[ ] RAW → waveform
[ ] RAW → video frames

## 9. Conversion Systems

### Audio → Image

[ ] Spectrogram mode
[ ] Waveform mode
[ ] FFT visualization
[ ] Frequency color mapping
[ ] Polar rendering
[ ] RGB mapping
[ ] Byte mapping
[ ] RAW reinterpretation
[ ] Scanline generation

### Image → Audio

[ ] Pixel synthesis
[ ] Spectrogram synthesis
[ ] Scanline synthesis
[ ] RGB channel synthesis
[ ] RAW PCM reinterpretation

### Video → Audio

[ ] Motion sonification
[ ] Frame synthesis
[ ] Brightness mapping
[ ] Optical flow synthesis
[ ] RAW reinterpretation

### Audio → Video

[ ] Audio reactive visuals
[ ] Waveform animation
[ ] Spectrogram animation

### Recursive Conversion

[ ] Recursive reinterpretation engine
[ ] Infinite loop safeguards

## 10. Filters & Effects

### Audio Effects

[ ] Reverb
[ ] Delay
[ ] Distortion
[ ] Bitcrush
[ ] EQ
[ ] Compression
[ ] Granular synthesis
[ ] Pitch shifting
[ ] Time stretching

### Visual Effects

[ ] Blur
[ ] Sharpen
[ ] Bloom
[ ] CRT
[ ] VHS
[ ] Scanlines
[ ] Chromatic aberration
[ ] Noise
[ ] Datamosh
[ ] Compression artifacts
[ ] Pixel drift
[ ] Color bleed
[ ] Tape damage
[ ] Analog distortion

### Binary Effects

[ ] Entropy injection
[ ] Probabilistic corruption
[ ] Byte swapping
[ ] Structure mutation
[ ] Bit flipping
[ ] Header destruction
[ ] Controlled offset corruption

## 11. Timeline System

### Tracks

[ ] Audio tracks
[ ] Video tracks
[ ] Image tracks
[ ] Effect tracks

### Editing

[ ] Trimming
[ ] Splitting
[ ] Keyframes
[ ] Automation curves
[ ] Snapping
[ ] Ripple editing

## 12. Real-Time Systems

### Live Inputs

[ ] Microphone input
[ ] Webcam input
[ ] Screen capture

#### Live Rendering

[ ] Real-time spectrogram
[ ] Live waveform
[ ] Live glitch effects
[ ] Audio reactive visuals
[ ] Live databending

## 13. Preset System

[ ] Save presets
[ ] Import presets
[ ] Export presets
[ ] Preset metadata
[ ] Preset thumbnails

## 14. Performance

### Optimization

[ ] SIMD acceleration
[ ] Multithreading
[ ] GPU acceleration
[ ] Async processing
[ ] Background rendering

### Stability

[ ] Memory leak testing
[ ] Stress testing
[ ] Large file testing
[ ] Corrupted file testing

## 15. GPU Features

### Rendering

[ ] OpenGL backend
[ ] Vulkan backend
[ ] Metal backend
[ ] wgpu abstraction

### Compute

[ ] GPU FFT
[ ] GPU filters
[ ] GPU spectrogram generation

## 16. Project System

### Projects

[ ] Create projects
[ ] Save projects
[ ] Load projects
[ ] Autosave
[ ] Recovery
[ ] Versioning

### Workspace

[ ] Layout persistence
[ ] Panel restoration
[ ] Session restore

## 17. Settings System

### User Settings

[ ] Themes
[ ] Keybinds
[ ] Audio settings
[ ] GPU settings
[ ] Export settings

### Profiles

[ ] Portable configs
[ ] Multiple profiles
[ ] Import/export settings

## 18. Themes

### Built-In Themes

[ ] AMOLED
[ ] Synthwave
[ ] CRT Green
[ ] Monochrome
[ ] Light theme

### Theme Engine

[ ] Custom themes
[ ] CSS-like styling
[ ] Runtime theme switching

## 19. CLI

### Core Commands

[ ] convert
[ ] export
[ ] inspect
[ ] render
[ ] benchmark

### Features

[ ] Batch mode
[ ] JSON output
[ ] Pipeline execution
[ ] Script automation

## 20. Security

[ ] Validate file parsing
[ ] Prevent arbitrary execution
[ ] Secure temp files

## 21. Packaging

### Distribution

[ ] AppImage
[ ] Flatpak
[ ] Nix package
[ ] Homebrew
[ ] Windows installer
[ ] DMG

### CI/CD

[ ] GitHub Actions
[ ] Automated releases
[ ] Nightly builds
[ ] Artifact signing

## 22. Future Experimental Features

[ ] Procedural media generation
[ ] Recursive conversion loops
[ ] Audio painting
[ ] Spectrogram painter
[ ] Binary camera
[ ] Live performance mode
[ ] Infinite reinterpretation mode
[ ] Live RAW visualizer
[ ] Entropy painter
[ ] Recursive databending chains
