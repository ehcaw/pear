import { useMicVAD } from "@ricky0123/vad-react";

const vad = useMicVAD({
  onSpeechEnd: (audio) => {
    console.log("user stopped speaking");
  },
});
