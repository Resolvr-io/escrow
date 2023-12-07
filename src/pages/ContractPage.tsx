import {
  Card,
  CardContent,
  CardHeader,
  CardFooter,
  CardTitle,
} from "~/components/ui/card";
import { useParams } from "react-router-dom";
import contracts from "~/static/contracts";
import { ArrowLeftIcon } from "@heroicons/react/24/outline";
import { SatoshiV2Icon } from "@bitcoin-design/bitcoin-icons-react/filled";
import { Link } from "react-router-dom";
import Stepper from "~/components/ui/stepper";
import { ContractStatus, StepperStatus } from "~/lib/constants";

export default function ContractPage() {
  const params = useParams();
  const contract = contracts.find(
    (contract) => contract.id === params.contractId,
  );
  if (!contract) {
    return <div>No Contract!</div>;
  }
  const {
    title,
    cost,
    description,
    status,
    expiration,
    createdOn,
    id,
    author,
  } = contract;
  const steps = [
    {
      id: ContractStatus.opened,
      name: "Contract Created",
      href: "#",
      status: StepperStatus.current,
    },
    {
      id: ContractStatus.sent,
      name: "Taker Applied",
      href: "#",
      status: StepperStatus.upcoming,
    },
    {
      id: ContractStatus.complete,
      name: "Contract Complete",
      href: "#",
      status: StepperStatus.upcoming,
    },
  ];

  // TODO: Find a better way to update the status
  switch (contract.status) {
    case ContractStatus.opened:
      steps.find((step) => step.id === contract.status)!.status =
        StepperStatus.complete;
      break;
    case ContractStatus.sent:
      steps.find((step) => step.id === contract.status)!.status =
        StepperStatus.complete;
      steps.find((step) => step.id === ContractStatus.opened)!.status =
        StepperStatus.complete;
      steps.find((step) => step.id === ContractStatus.complete)!.status =
        StepperStatus.upcoming;
      break;
    case ContractStatus.complete:
      steps.find((step) => step.id === contract.status)!.status =
        StepperStatus.complete;
      steps.find((step) => step.id === ContractStatus.opened)!.status =
        StepperStatus.complete;
      steps.find((step) => step.id === ContractStatus.sent)!.status =
        StepperStatus.complete;
      break;
    default:
      break;
  }

  return (
    <div className="flex-1 flex-col space-y-8 p-8 pt-6 md:flex">
      <Link
        to="/"
        className="flex w-48 items-center gap-x-2 rounded-lg bg-gray-50 px-3 py-2 text-sm font-medium text-gray-800 shadow-lg shadow-gray-900/5 ring-1 ring-gray-300 hover:bg-white dark:bg-gray-800 dark:text-gray-200 dark:ring-gray-800 dark:hover:bg-gray-700/50"
      >
        <ArrowLeftIcon className="h-4 w-4" />
        <span>Back to all contracts</span>
      </Link>

      <div>
        <Stepper steps={steps} />
      </div>
      <div className="flex text-2xl">
        <SatoshiV2Icon style={{ height: "2rem", width: "2rem" }} />
        {Number(cost).toLocaleString()}
      </div>

      <div>by {author}</div>
      <Card>
        <CardHeader>
          <CardTitle>{title}</CardTitle>
        </CardHeader>
        <CardContent>
          <div>{description}</div>
        </CardContent>
        <CardFooter>
          <div className="flex w-full flex-row justify-between text-gray-500">
            <div>Created on {createdOn}</div>
            <div>Expires on {expiration}</div>
          </div>
        </CardFooter>
      </Card>
    </div>
  );
}
