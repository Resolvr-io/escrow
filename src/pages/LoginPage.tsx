import LoginForm from "~/components/login/LoginForm";
import { useState } from "react";
// import RegisterForm from "~/components/login/RegisterForm";

export default function LoginPage() {
  const [formState, setFormState] = useState<"login" | "register">("login");

  return (
    <div className="-mt-28 flex h-screen items-center justify-center">
      {formState === "login" && <LoginForm setFormState={setFormState} />}
      {/*TODO: implement register form for new users*/}
      {/* {formState === "register" && <RegisterForm setFormState={setFormState} />} */}
    </div>
  );
}
