import React from "react";
import ReactDOM from "react-dom/client";
import App from "~/App";
import "~/styles/globals.css";
import { BrowserRouter } from "react-router-dom";
import { Toaster } from "~/components/ui/toaster";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
      <Toaster />
    </BrowserRouter>
  </React.StrictMode>,
);
