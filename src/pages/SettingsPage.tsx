import UserAvatar from "~/components/profile/UserAvatar";
import { Profile } from "~/types";
import { ArrowPathIcon, UserCircleIcon } from "@heroicons/react/24/outline";
import { useEffect, useState } from "react";
import { Event, getEventHash, getSignature, nip19 } from "nostr-tools";
import { pc, publish } from "~/lib/nostr";
import useEventStore from "~/stores/eventStore";
import useAuthStore from "~/stores/authStore";

import { invoke } from "@tauri-apps/api";
import { RELAYS } from "~/lib/constants";
import { Button } from "~/components/ui/button";

import { useToast } from "~/components/ui/use-toast";
import { fetchProfileEvent } from "~/lib/auth";

export default function SettingsPage() {
  const [profile, setProfile] = useState<Profile>(pc(null));
  const { profileMap, addProfile } = useEventStore();
  const { pubkey } = useAuthStore();
  const { toast } = useToast();

  async function getNsec() {
    const npub = nip19.npubEncode(pubkey);
    const nsec: string = await invoke("get_nsec", {
      npub: npub,
    });
    return nsec;
  }

  useEffect(() => {
    if (!pubkey) {
      return;
    }

    if (!profileMap[pubkey]) {
      return;
    }

    setProfile(pc(profileMap[pubkey]));
  }, [pubkey]);

  const updateProfile = async () => {
    if (!pubkey) {
      return;
    }

    const updatedProfile = {
      ...pc(profileMap[pubkey]),
      ...profile,
    };

    let tags: string[][] = [];

    if (profileMap) {
      const oldProfile = profileMap[pubkey];
      if (oldProfile) {
        tags = oldProfile.tags;
      }
    }

    let event: Event = {
      id: "",
      sig: "",
      kind: 0,
      created_at: Math.floor(Date.now() / 1000),
      tags: tags,
      content: JSON.stringify(updatedProfile),
      pubkey: pubkey,
    };

    event.id = getEventHash(event);

    const nsec = await getNsec();
    const sk = nip19.decode(nsec).data as string;
    event.sig = getSignature(event, sk);

    const onSeen = (event: Event) => {
      addProfile(pubkey, event);
      toast({
        title: "Profile updated",
        description: "Your profile has been updated.",
      });
    };

    publish(RELAYS, event, onSeen);
  };

  const handleSubmit = (e: any) => {
    e.preventDefault();
    updateProfile();
  };

  const refreshAccount = () => {
    fetchProfileEvent(pubkey);
  };

  return (
    <div className="flex w-full flex-col items-center justify-center px-4 pb-24 pt-10 sm:px-0">
      <form className="w-full" onSubmit={handleSubmit}>
        <div className="space-y-12">
          <div className="border-b border-zinc-900/10 pb-12 dark:border-zinc-700">
            <div className="flex items-center gap-x-2">
              <h2 className="text-lg font-semibold leading-7 text-zinc-900 dark:text-zinc-100">
                account settings
              </h2>
              <button type="button" onClick={refreshAccount}>
                <ArrowPathIcon className="h-4 w-4 text-zinc-900 dark:text-zinc-400" />
              </button>
            </div>
            <div className="mt-10 flex flex-col gap-x-6 gap-y-8">
              <div className="mt-2 flex items-center gap-x-3">
                {pubkey ? (
                  <UserAvatar pubkey={pubkey} />
                ) : (
                  <UserCircleIcon
                    className="h-12 w-12 text-zinc-300 dark:text-zinc-600"
                    aria-hidden="true"
                  />
                )}

                <div className="flex w-full max-w-sm rounded-md shadow-sm ring-1 ring-inset ring-zinc-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-gray-600 dark:ring-zinc-600">
                  <input
                    name="picture"
                    type="text"
                    className="block w-full rounded-md border-0 py-1.5 text-zinc-900 shadow-sm ring-1 ring-inset ring-zinc-300 placeholder:text-zinc-400 focus:ring-2 focus:ring-inset focus:ring-gray-600 dark:bg-zinc-800 dark:text-zinc-100 dark:ring-zinc-600 sm:text-sm sm:leading-6"
                    placeholder="image URL"
                    defaultValue={profile.picture}
                    onChange={(e) => {
                      setProfile({
                        ...profile,
                        picture: e.target.value,
                      });
                    }}
                  />
                </div>
              </div>
              <div>
                <label
                  htmlFor="username"
                  className="block text-sm font-medium leading-6 text-zinc-900 dark:text-zinc-100"
                >
                  username
                </label>
                <div className="mt-2">
                  <div className="flex rounded-md shadow-sm ring-1 ring-inset ring-zinc-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-gray-600 dark:ring-zinc-600 sm:max-w-md">
                    <input
                      name="username"
                      type="text"
                      id="username"
                      autoComplete="username"
                      className="block w-full rounded-md border-0 py-1.5 text-zinc-900 shadow-sm ring-1 ring-inset ring-zinc-300 placeholder:text-zinc-400 focus:ring-2 focus:ring-inset focus:ring-gray-600 dark:bg-zinc-800 dark:text-zinc-100 dark:ring-zinc-600 sm:text-sm sm:leading-6"
                      defaultValue={profile.name}
                      onChange={(e) => {
                        setProfile({
                          ...profile,
                          name: e.target.value,
                        });
                      }}
                    />
                  </div>
                </div>
              </div>
              <div>
                <label
                  htmlFor="about"
                  className="block text-sm font-medium leading-6 text-zinc-900 dark:text-zinc-100"
                >
                  about
                </label>
                <div className="mt-2">
                  <div className="flex rounded-md shadow-sm ring-1 ring-inset ring-zinc-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-gray-600 dark:ring-zinc-600 sm:max-w-md">
                    <input
                      name="about"
                      type="text"
                      id="about"
                      className="block w-full rounded-md border-0 py-1.5 text-zinc-900 shadow-sm ring-1 ring-inset ring-zinc-300 placeholder:text-zinc-400 focus:ring-2 focus:ring-inset focus:ring-gray-600 dark:bg-zinc-800 dark:text-zinc-100 dark:ring-zinc-600 sm:text-sm sm:leading-6"
                      defaultValue={profile.about}
                      onChange={(e) => {
                        setProfile({
                          ...profile,
                          about: e.target.value,
                        });
                      }}
                    />
                  </div>
                </div>
              </div>

              <div>
                <label
                  htmlFor="lud16"
                  className="block text-sm font-medium leading-6 text-zinc-900 dark:text-zinc-100"
                >
                  lightning address
                </label>
                <div className="mt-2">
                  <div className="flex rounded-md shadow-sm ring-1 ring-inset ring-zinc-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-gray-600 dark:ring-zinc-600 sm:max-w-md">
                    <input
                      name="lud16"
                      type="text"
                      id="lud16"
                      autoComplete="lud16"
                      className="block w-full rounded-md border-0 py-1.5 text-zinc-900 shadow-sm ring-1 ring-inset ring-zinc-300 placeholder:text-zinc-400 focus:ring-2 focus:ring-inset focus:ring-gray-600 dark:bg-zinc-800 dark:text-zinc-100 dark:ring-zinc-600 sm:text-sm sm:leading-6"
                      defaultValue={profile.lud16}
                      onChange={(e) => {
                        setProfile({
                          ...profile,
                          lud16: e.target.value,
                        });
                      }}
                    />
                  </div>
                </div>
              </div>

              <div>
                <label
                  htmlFor="nip05"
                  className="block text-sm font-medium leading-6 text-zinc-900 dark:text-zinc-100"
                >
                  nip05
                </label>
                <div className="mt-2">
                  <div className="flex rounded-md shadow-sm ring-1 ring-inset ring-zinc-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-gray-600 dark:ring-zinc-600 sm:max-w-md">
                    <input
                      name="nip05"
                      type="text"
                      id="nip05"
                      autoComplete="nip05"
                      className="block w-full rounded-md border-0 py-1.5 text-zinc-900 shadow-sm ring-1 ring-inset ring-zinc-300 placeholder:text-zinc-400 focus:ring-2 focus:ring-inset focus:ring-gray-600 dark:bg-zinc-800 dark:text-zinc-100 dark:ring-zinc-600 sm:text-sm sm:leading-6"
                      defaultValue={profile.nip05}
                      onChange={(e) => {
                        setProfile({
                          ...profile,
                          nip05: e.target.value,
                        });
                      }}
                    />
                  </div>
                </div>
              </div>

              <div>
                <label
                  htmlFor="website"
                  className="block text-sm font-medium leading-6 text-zinc-900 dark:text-zinc-100"
                >
                  website
                </label>
                <div className="mt-2">
                  <div className="flex rounded-md shadow-sm ring-1 ring-inset ring-zinc-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-gray-600 dark:ring-zinc-600 sm:max-w-md">
                    <input
                      type="text"
                      name="website"
                      id="website"
                      autoComplete="website"
                      className="block w-full rounded-md border-0 py-1.5 text-zinc-900 shadow-sm ring-1 ring-inset ring-zinc-300 placeholder:text-zinc-400 focus:ring-2 focus:ring-inset focus:ring-gray-600 dark:bg-zinc-800 dark:text-zinc-100 dark:ring-zinc-600 sm:text-sm sm:leading-6"
                      defaultValue={profile.website}
                      onChange={(e) => {
                        setProfile({
                          ...profile,
                          website: e.target.value,
                        });
                      }}
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="mt-6 flex items-center justify-start">
          <Button type="submit">Submit</Button>
        </div>
      </form>
    </div>
  );
}
