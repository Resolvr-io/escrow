import { Route, Routes } from "react-router-dom";
import ContractsPage from "./pages/ContractsPage";
import CreateContractPage from "./pages/CreateContractPage";
import ProtectedRoute from "./components/auth/ProtectedRoute"; // Import your ProtectedRoute component
import LoginPage from "./pages/LoginPage";
import ContractPage from "./pages/ContractPage";
import Header from "./components/header/header";

export default function DashboardPage() {
  return (
    <>
      <Header></Header>
      <Routes>
        {/* <Route path="login" element={<CreateContract />} /> */}
        <Route path="/" element={<ContractsPage />} />
        <Route path="contracts/:contractId" element={<ContractPage />} />
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
