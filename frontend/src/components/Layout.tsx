import { type ParentComponent } from "solid-js";
import Navbar from "./Navbar";

const Layout: ParentComponent = (props) => {
  return (
    <div style={{ "min-height": "100vh", display: "flex", "flex-direction": "column" }}>
      <Navbar />
      <main style={{ flex: "1", padding: "1rem" }}>
        {props.children}
      </main>
    </div>
  );
};

export default Layout;
