import { useParams } from "react-router-dom";
import { IContract } from "~/lib/constants";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "~/components/ui/card";
interface Props {
  contract: IContract;
}
import { SatoshiV2Icon } from "@bitcoin-design/bitcoin-icons-react/filled";
export default function Contract({ contract }: Props) {
  const {
    title,
    description,
    status,
    expiration,
    cost,
    createdOn,
    id,
    author,
  } = contract;
  return (
    <div>
      <Card className="hover:border-gray-400/70 ">
        <CardHeader className="pl-3">
          <div className="flex text-2xl">
            <SatoshiV2Icon style={{ height: "2rem", width: "2rem" }} />
            {Number(cost).toLocaleString()}
            <div className="ml-auto inline-flex items-center gap-x-3 rounded-md bg-white px-3 py-2 text-sm font-medium text-gray-600 ring-2 ring-inset ring-gray-300 dark:bg-gray-900 dark:text-white dark:ring-gray-800">
              <span>{status}</span>
              <svg
                className="h-2 w-2 fill-yellow-400"
                viewBox="0 0 6 6"
                aria-hidden="true"
              >
                <circle cx={3} cy={3} r={3} />
              </svg>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <CardTitle className="mb-4">
            <div>{title}</div>
          </CardTitle>
          <div>{description}</div>
        </CardContent>
        <CardFooter>
          <div className="flex w-full flex-row justify-between text-gray-500">
            <div>
              Created by {author} on: {createdOn}
            </div>
            <div>Expires on {expiration}</div>
          </div>
        </CardFooter>
      </Card>
    </div>
  );
}
