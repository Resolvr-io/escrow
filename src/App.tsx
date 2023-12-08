import { Route, Routes } from "react-router-dom";
import ContractsPage from "~/pages/ContractsPage";
import CreateContractPage from "~/pages/CreateContractPage";
import ProtectedRoute from "~/components/auth/ProtectedRoute";
import LoginPage from "~/pages/LoginPage";
import ContractPage from "~/pages/ContractPage";
import Header from "~/components/header/Header";
import { useEffect } from "react";
import { restoreLogin } from "~/lib/auth";
import SettingsPage from "./pages/SettingsPage";
import BitconNodePage from "./pages/BitcoinNodePage";

export default function DashboardPage() {
  useEffect(() => {
    restoreLogin();
  }, []);

  return (
    <>
      <Header />
      <Routes>
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
        <Route path="/node" element={<BitconNodePage />} />
        <Route
          path="/settings"
          element={
            <ProtectedRoute>
              <SettingsPage />
            </ProtectedRoute>
          }
        />
      </Routes>
    </>
  );
}
