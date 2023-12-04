import { Route, Routes } from "react-router-dom";
import ContractsPage from "~/pages/ContractsPage";
import CreateContractPage from "~/pages/CreateContractPage";
import ProtectedRoute from "~/components/auth/ProtectedRoute";
import LoginPage from "~/pages/LoginPage";
import ContractPage from "~/pages/ContractPage";
import Header from "~/components/header/Header";

export default function DashboardPage() {
  return (
    <>
      <Header />
      <Routes>
        <Route path="/" element={<ContractsPage />} />
        <Route path="contracts/:contractId" element={<ContractPage />} />
        <Route path="create" element={<CreateContractPage />} />
        <Route path="/login" element={<LoginPage />} />
      </Routes>
    </>
  );
}
