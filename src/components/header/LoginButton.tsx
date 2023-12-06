import { Link } from "react-router-dom";
import { Button } from "../ui/button";

export default function LoginButton() {
  return (
    <Button asChild variant="outline">
      <Link to="/login">login &rarr;</Link>
    </Button>
  );
}
