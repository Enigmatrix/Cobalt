using System;
using System.Diagnostics;
using System.Runtime.CompilerServices;
using System.Threading.Tasks;
using Grpc.Core;
using Vanara.Extensions;
using Vanara.PInvoke;

namespace Controllable.Common
{
    public abstract class ControllableBase : Controllable.ControllableBase
    {
        public override Task<Empty> SetNtPath(StringValue request, ServerCallContext context)
        {
            var process = (HPROCESS)Process.GetCurrentProcess().Handle;
            var pbi = NtDll.NtQueryInformationProcess<NtDll.PROCESS_BASIC_INFORMATION>(process,
                NtDll.PROCESSINFOCLASS.ProcessBasicInformation);
            var peb = pbi.AsRef().PebBaseAddress.ToStructure<NtDll.PEB>();
            var param = peb.ProcessParameters.ToStructure<NtDll.RTL_USER_PROCESS_PARAMETERS>();
            param.ImagePathName = FfiUnicodeString.FromString(request.Inner);
            return Task.FromResult(new Empty());
        }

        public override Task<Empty> SetNtCmdLine(StringValue request, ServerCallContext context)
        {
            var process = (HPROCESS)Process.GetCurrentProcess().Handle;
            var pbi = NtDll.NtQueryInformationProcess<NtDll.PROCESS_BASIC_INFORMATION>(process,
                NtDll.PROCESSINFOCLASS.ProcessBasicInformation);
            var peb = pbi.AsRef().PebBaseAddress.ToStructure<NtDll.PEB>();
            var param = peb.ProcessParameters.ToStructure<NtDll.RTL_USER_PROCESS_PARAMETERS>();
            param.CommandLine = FfiUnicodeString.FromString(request.Inner);
            return Task.FromResult(new Empty());
        }

        public override Task<Empty> SetForeground(HwndValue request, ServerCallContext context)
        {
            // User32.AllowSetForegroundWindow(ASFW_ANY)
            User32.SetForegroundWindow(new HWND(new IntPtr((int)request.Hwnd)));
            return Task.FromResult(new Empty());
        }
    }
}
