import { createRoot } from "react-dom/client";
import DemoTransform from "./DemoTransform.jsx";

const INPUT = `BEGIN:VCARD
VERSION:4.0
FN:Alex Rivera
TEL;TYPE=cell:+1-555-0191
EMAIL:alex.rivera@gmail.com
ADR;TYPE=home:;;482 Birch Ln;Madison;WI;53703;US
NOTE:Referred by Dr. Chen — follow up on referral
END:VCARD`;

createRoot(document.getElementById("personal-demo")).render(
  <DemoTransform
    inputVcard={INPUT}
    defaultProps="FN,TEL"
    accentLabel="field redaction"
  />
);
