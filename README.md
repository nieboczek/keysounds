# KeySounds
Play a sound entirely using your keyboard to two output devices.
Made to be used in a Voice Channel with friends.

# TODO
- Shit mic mode (Old Python implementation below)
```py
    def mic_loop(self) -> None:
        mic_index = self.get_device_index(MIC_DEVICE_NAME)
        virtual_mic_index = self.get_device_index(VIRTUAL_MIC_NAME)

        FORMAT = pyaudio.paInt16
        CHANNELS = 1
        RATE = 44100
        CHUNK = 1024

        p = pyaudio.PyAudio()

        mic_stream = p.open(format=FORMAT, channels=CHANNELS, rate=RATE, input=True,
                            frames_per_buffer=CHUNK, input_device_index=mic_index)
        output_stream = p.open(format=FORMAT, channels=CHANNELS, rate=RATE, output=True,
                               frames_per_buffer=CHUNK, output_device_index=virtual_mic_index)

        while True:
            try:
                mic_data = mic_stream.read(CHUNK, exception_on_overflow=False)
                mic_array = np.frombuffer(mic_data, dtype=np.int16)

                if self.shit_mic:
                    # Aggressive bitcrushing (drop lower 5 bits)
                    mic_array = (mic_array >> 5) << 5

                    abs_mic = np.abs(mic_array)
                    gain_boost = np.where(abs_mic < 2000, 3, 1).astype(np.int16)
                    mic_array = mic_array * gain_boost

                    # Heavy overdrive + hard clipping
                    mic_array = mic_array * 5
                    mic_array = np.clip(mic_array, -10000, 10000).astype(np.int16)
                    mic_array = mic_array // 2  # quieter it

                output_stream.write(mic_array.tobytes())
            except Exception as e:
                self.logn(f"Error in mic loop: {e}")
                break

        mic_stream.stop_stream()
        mic_stream.close()
        output_stream.stop_stream()
        output_stream.close()
        p.terminate()
```
- Focus console on Ctrl+Alt+T
- Player state rendering
- Random audio triggering
- Built-in config editor
- Upgrade the `load_and_play` function
