using System;
using System.Diagnostics;
using System.Linq;
using System.Runtime.CompilerServices;
using Controllable.Common;
using Vanara.Extensions;
using Vanara.InteropServices;
using Vanara.PInvoke;
using Xunit;

namespace Controllable.VerficationTests
{
    public class NativeTests
    {

        [Fact]
        public void Blank()
        {
            Assert.True(true);
        }

        [Fact]
        public void What()
        {

			/*var curProc = Process.GetCurrentProcess();
			HPROCESS hProc = curProc.Handle;

            NtDll.NtQueryResult<NtDll.PROCESS_BASIC_INFORMATION> pbi = null;
            pbi = NtDll.NtQueryInformationProcess<NtDll.PROCESS_BASIC_INFORMATION>(hProc,
                NtDll.PROCESSINFOCLASS.ProcessBasicInformation);
			// Can do AsRef here since PROCESS_BASIC_INFORMATION has no managed types
			ref var rpbi = ref pbi.AsRef();
			Assert.Equal(rpbi.UniqueProcessId.ToInt32(), curProc.Id);
			// Have to use ToStructure here since PEB has managed types
			var peb = rpbi.PebBaseAddress.ToStructure<NtDll.PEB>();
			// Have to use ToStructure here since RTL_USER_PROCESS_PARAMETERS has managed types
			var upp = peb.ProcessParameters.ToStructure<NtDll.RTL_USER_PROCESS_PARAMETERS>();
			Trace.WriteLine($"Img: {upp.ImagePathName}; CmdLine: {upp.CommandLine}");*/
        }

        [Fact]
        public void ChangeImagePathName()
        {
            var process = Process.GetProcesses().First(x => x.ProcessName.Contains("devenv"));
            using var hProc = Kernel32.OpenProcess((uint)(Kernel32.ProcessAccess.PROCESS_QUERY_INFORMATION | Kernel32.ProcessAccess.PROCESS_VM_READ), false, (uint)process.Id);

            var pbi = NtDll.NtQueryInformationProcess<NtDll.PROCESS_BASIC_INFORMATION>(hProc,
                NtDll.PROCESSINFOCLASS.ProcessBasicInformation);
            using var pebPtr = new SafeHGlobalStruct<NtDll.PEB>();
            Kernel32.ReadProcessMemory(hProc, pbi.AsRef().PebBaseAddress, pebPtr, pebPtr.Size, out _);
            using var rtlUserParamsPtr = new SafeHGlobalStruct<NtDll.RTL_USER_PROCESS_PARAMETERS>();
            Kernel32.ReadProcessMemory(hProc, pebPtr.Value.ProcessParameters, rtlUserParamsPtr, rtlUserParamsPtr.Size,
                out _);

            var path = GetString(rtlUserParamsPtr.Value.ImagePathName);
            var cmd = GetString(rtlUserParamsPtr.Value.CommandLine);

			string GetString(in NtDll.UNICODE_STRING us)
			{
				using var mem = new SafeCoTaskMemString(us.MaximumLength);
                Kernel32.ReadProcessMemory(hProc, us.Buffer, mem, mem.Size, out _);
				return mem;
			}
            //Assert.Equal("", path);*

            unsafe
            {
                /*var pbi = NtDll.NtQueryInformationProcess<NtDll.PROCESS_BASIC_INFORMATION>(process,
                    NtDll.PROCESSINFOCLASS.ProcessBasicInformation);
                ref var peb = ref Unsafe.AsRef<NtDll.PEB>((void*)pbi.AsRef().PebBaseAddress);
                ref var param = ref Unsafe.AsRef<NtDll.RTL_USER_PROCESS_PARAMETERS>((void*)peb.ProcessParameters);
                var w = FfiUnicodeString.FromString("WHAT");
                param.ImagePathName = w;*/

                /*var pbi2 = NtDll.NtQueryInformationProcess<NtDll.PROCESS_BASIC_INFORMATION>(process,
                    NtDll.PROCESSINFOCLASS.ProcessBasicInformation);
                var peb2 = pbi2.AsRef().PebBaseAddress.ToStructure<NtDll.PEB>();
                var param2 = peb2.ProcessParameters.ToStructure<NtDll.RTL_USER_PROCESS_PARAMETERS>();
                //Assert.Equal(param.ImagePathName.Buffer, param2.ImagePathName.Buffer);
                Assert.Equal(IntPtr.Zero, param2.ImagePathName.Buffer);*/
                //Assert.True(true);
                
            }

        }
    }
}
