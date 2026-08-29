import { createRoot } from "react-dom/client";
import DemoTransform from "./DemoTransform.jsx";

const INPUT = `BEGIN:VCARD
VERSION:4.0
FN:Jamie Morgan
ORG:Option63 Ltd.
TITLE:Account Manager
TEL;TYPE=cell:+1-555-0142
EMAIL:jmorgan@corp.example
ADR;TYPE=work:;;100 Harbor Ave;Portland;OR;97201;US
NOTE:Calls contract renewal — $240k ARR, renewal 09/15
END:VCARD`;

createRoot(document.getElementById("shadow-it-demo")).render(
  <DemoTransform
    inputVcard={INPUT}
    defaultProps="FN,TEL"
    accentLabel="property allow-listing"
  />
);
