using BenchmarkDotNet.Attributes;

namespace Cobalt.Tests.Benchmarks
{
    [MemoryDiagnoser]
    public class InsertAppBenchmarks
    {
        [GlobalSetup]
        public void OpenConnection()
        {
        }

        [GlobalCleanup]
        public void Close()
        {
        }

        [Benchmark(Baseline = true)]
        public void AddAppUsingRaw2()
        {
        }

        [Benchmark]
        public void AddAppUsingDapper()
        {
        }

        [Benchmark]
        public void AddAppUsingRepo()
        {
        }
    }
}