import React from "react";
import { Navigate, useLocation } from "react-router-dom";
import useAuthStore from "~/stores/authStore";

type Props = {
  children: React.ReactNode;
};

export default function ProtectedRoute({ children }: Props) {
  const { pubkey } = useAuthStore();
  const location = useLocation();

  if (!pubkey || pubkey === "") {
    // Redirect them to the /login page, but save the current location they were
    // trying to go to when they were redirected. This allows us to send them
    // along to that location after they login.
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  return children;
}
