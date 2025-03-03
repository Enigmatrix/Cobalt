import { CreateAlertForm } from "@/components/alert/create-alert-form";

export default function CreateAlerts() {
  return (
    <>
      <main>
        <CreateAlertForm onSubmit={console.log} />
      </main>
    </>
  );
}
