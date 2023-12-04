import { Button } from "~/components/ui/button";
import Contracts from "~/components/contracts/contracts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";

// import { invoke } from "@tauri-apps/api/tauri";

// import { invoke } from "@tauri-apps/api/primitives";

import {
  createBrowserRouter,
  createRoutesFromElements,
  Link,
  Route,
  RouterProvider,
  Routes,
} from "react-router-dom";

// import { invoke } from "@tauri-apps/api/tauri";

async function setSecret() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  // const test = await invoke("set_secret");
  // console.log(test);
  // alert("hello");
}

export default function ContractsPage() {
  return (
    <>
      <div className="flex-col md:flex">
        <div className="flex-1 space-y-4 p-8 pt-6">
          <div className="flex items-center justify-between space-y-2">
            <h2 className="text-3xl font-bold tracking-tight">Contracts</h2>
            <div className="flex items-center space-x-2">
              {/* <CalendarDateRangePicker /> */}
              <Button

              // onClick={setSecret}
              >
                <Link to="/create">Create Contract</Link>
              </Button>
            </div>
          </div>
          <Tabs defaultValue="overview" className="space-y-4">
            <TabsList>
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="sent">Sent</TabsTrigger>
              <TabsTrigger value="received">Received</TabsTrigger>
              {/* <TabsTrigger value="accepted">Accepted</TabsTrigger> */}
              {/* <TabsTrigger value="rejected">Rejected</TabsTrigger> */}
              <TabsTrigger value="archive">Complete</TabsTrigger>
            </TabsList>
            <TabsContent value="overview" className="space-y-4">
              <div className="flex flex-col">
                <Contracts />
              </div>
            </TabsContent>
            <TabsContent value="sent" className="space-y-4">
              <div className="flex flex-col">
                {/* this is list of cards*/}
                <Card>
                  <CardHeader>
                    <CardTitle>Overview</CardTitle>
                  </CardHeader>
                  <CardContent className="pl-2">
                    sent
                    {/* <Overview /> */}
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </>
  );
}
