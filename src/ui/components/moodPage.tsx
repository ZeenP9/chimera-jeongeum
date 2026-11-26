import { roomDecaySeconds } from "../../audio/AudioEngine.ts";
import { inPlugin } from "../../audio/pluginBridge.ts";
import { useStore } from "../store.ts";
import type { EncoderPage } from "./Encoder.tsx";
import { JeongeumSwitch } from "./JeongeumSwitch.tsx";

/**
 * The Mood page of the Sing and Play encoders: a room after the voice, which the engine makes dry, on
 * the green, red and white slots, and 正音's four-way switch beside the white one.
 * Dry/Wet is how much of the room is mixed in; Size goes from a small room to a large hall (how far the
 * walls are); Decay is how live a room of that size is, shown as its decay time. None in the plugin:
 * the host adds its own room.
 */
export function useMoodPage(): EncoderPage[] {
  const room = useStore((s) => s.room);
  const size = useStore((s) => s.roomSize);
  const decay = useStore((s) => s.roomDecay);
  const setRoom = useStore((s) => s.setRoom);
  const setRoomShape = useStore((s) => s.setRoomShape);
  if (inPlugin) return [];
  return [
    {
      name: "Mood",
      encoders: [
        { label: "Dry/Wet", value: room, min: 0, max: 0.5, onChange: setRoom, format: (v) => ({ value: String(Math.round(v * 100)), unit: "%" }) },
        {
          label: "Size",
          value: size,
          min: 0,
          max: 1,
          onChange: (v) => setRoomShape({ roomSize: Math.round(v * 20) / 20 }),
          format: (v) => ({ value: String(Math.round(v * 100)), unit: v < 0.3 ? "room" : v < 0.7 ? "hall" : "large hall" }),
        },
        {
          label: "Decay",
          value: decay,
          min: 0,
          max: 1,
          onChange: (v) => setRoomShape({ roomDecay: Math.round(v * 20) / 20 }),
          format: (v) => ({ value: roomDecaySeconds(size, v).toFixed(2), unit: "s" }),
          slot: 3, // white: the three knobs are green, red, white (no pink)
        },
      ],
      extra: <JeongeumSwitch />,
    },
  ];
}
