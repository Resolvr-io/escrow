import { Route, Routes } from "react-router-dom";
import ContractsPage from "./pages/ContractsPage";
import CreateContractPage from "./pages/CreateContractPage";
import ProtectedRoute from "./components/auth/ProtectedRoute"; // Import your ProtectedRoute component
import LoginPage from "./pages/LoginPage";

export default function DashboardPage() {
  return (
    <>
      <Routes>
        {/* <Route path="login" element={<CreateContract />} /> */}
        <Route path="/" element={<ContractsPage />} />
        <Route
          path="create"
          element={
            <ProtectedRoute>
              <CreateContractPage />
            </ProtectedRoute>
          }
        />
        <Route path="/login" element={<LoginPage />} />
      </Routes>
    </>
  );
}
