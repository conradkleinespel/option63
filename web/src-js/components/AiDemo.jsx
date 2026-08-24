import { createRoot } from "react-dom/client";
import DemoTransform from "./DemoTransform.jsx";

const INPUT = `BEGIN:VCARD
VERSION:4.0
FN:Sarah Nguyen
ORG:Acme Corp
TITLE:CTO
TEL;TYPE=cell:+1-555-0137
EMAIL:sarah.nguyen@acme.example
ADR;TYPE=work:;;1 Innovation Dr;Austin;TX;78701;US
NOTE:Evaluating our product — sent NDA 06/12
END:VCARD`;

createRoot(document.getElementById("ai-demo")).render(
  <DemoTransform
    inputVcard={INPUT}
    defaultProps="FN,TITLE"
    accentLabel="minimal AI-safe subset"
  />
);
